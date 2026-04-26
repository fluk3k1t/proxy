use std::{
    collections::HashMap,
    fs,
    io::{Error, ErrorKind},
    iter::Map,
    time::Duration,
};

use async_from::{AsyncFrom, AsyncInto, async_trait};
use bytes::{Bytes, BytesMut};
// use futures_core::Stream;
// use futures_util::StreamExt;
use httparse::{EMPTY_HEADER, Request, Status};
use rcgen::{
    CertificateParams, CertifiedKey, Issuer, KeyPair, SigningKey, generate_simple_self_signed,
};
use reqwest::Response;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter},
    net::{TcpListener, TcpStream},
    time::{error::Elapsed, timeout},
};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed, FramedRead};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = TcpListener::bind("127.0.0.1:8000").await?;

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                let mut conn = Connection::new(socket);
                tokio::spawn(async move { conn.handle_connection().await });
            }
            Err(e) => println!("couldn't get client: {:?}", e),
        }
    }

    Ok(())
}

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection { stream }
    }

    async fn handle_connection(&mut self) -> Result<(), Error> {
        let mut packet_reader = Framed::new(&mut self.stream, HttpDecoder::default());

        while let Some(r) = packet_reader.next().await {
            match r {
                Ok(packet) => {
                    self.handle_packet(packet).await;
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    async fn handle_packet(&mut self, packet: Packet) {
        if let Some(method) = packet.method {
            // let host = packet.headers.get("host").unwrap();
            // println!("{} {:?}", method, packet.headers);

            match &method[..] {
                "GET" => {
                    let response = reqwest::get(packet.path.unwrap()).await.unwrap();
                    let bytes = response_to_bytes(response).await.unwrap();

                    self.stream.write_all(&bytes).await.unwrap();
                }
                "CONNECT" => {}
                _ => {}
            }
        } else {
            panic!("method does not exist!");
        }
    }
}

async fn response_to_bytes(resp: reqwest::Response) -> Result<Bytes, reqwest::Error> {
    let status = resp.status();
    let headers = resp.headers().clone();
    let body = resp.text().await.unwrap();

    let mut presp = format!("HTTP/1.1 {}\r\n", status);
    for (n, v) in headers.iter() {
        let v = String::from_utf8_lossy(v.as_bytes());
        presp.push_str(&format!("{}: {}\r\n", n, v));
    }
    presp.push_str("\r\n");

    if headers.iter().any(|(h, v)| h == "transfer-encoding") {
        presp.push_str(&format!("{:x}\r\n{}", body.len(), body));
    } else {
        presp.push_str(body.as_str());
    }

    Ok(Bytes::from(presp))
}

#[derive(Debug)]
pub struct Packet {
    headers: HashMap<String, String>,
    method: Option<String>,
    path: Option<String>,
}

pub struct HttpDecoder {}

impl Default for HttpDecoder {
    fn default() -> Self {
        HttpDecoder {}
    }
}

impl Decoder for HttpDecoder {
    type Error = std::io::Error;
    type Item = Packet;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // プロキシに関連するHTTPメソッドはボディを持たないはずなのでヘッダーのみを読む
        // ボディが存在して、かつパケットが分割されて送られてくると次回以降のパケットの読み込みに支障をきたす可能性はある
        if src.windows(4).any(|w| w == b"\r\n\r\n") {
            let mut headers = [EMPTY_HEADER; 64];
            let mut req = Request::new(&mut headers);

            // println!("{:?}", String::from_utf8_lossy(src));

            match req
                .parse(src)
                .map_err(|_| std::io::Error::new(ErrorKind::Other, "oh no!"))?
            {
                Status::Partial => Ok(None),
                Status::Complete(n) => {
                    let headers = req
                        .headers
                        .iter()
                        .map(|h| {
                            (
                                h.name.to_owned().to_lowercase(),
                                String::from_utf8_lossy(h.value).to_string(),
                            )
                        })
                        .collect();

                    let method = req.method.map(String::from);
                    let path = req.path.map(String::from);

                    src.clear();

                    Ok(Some(Packet {
                        headers,
                        method,
                        path,
                    }))
                }
            }
        } else {
            Ok(None)
        }
    }
}
