use std::{
    collections::HashMap,
    fs,
    io::{Error, ErrorKind},
    iter::Map,
    sync::Arc,
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
use reqwest::{Identity, Response};
// use rustls_pki_types::ServerName;
// use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter},
    net::TcpListener,
    time::{error::Elapsed, timeout},
};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        ClientConfig, RootCertStore, ServerConfig, ServerConnection, Stream,
        pki_types::{PrivateKeyDer, pem::PemObject},
    },
};
use tokio_rustls::{TlsConnector, rustls::pki_types::CertificateDer};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed, FramedRead};

#[derive(Debug)]
pub struct Packet {
    pub headers: HashMap<String, String>,
    pub method: Option<String>,
    pub path: Option<String>,
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
