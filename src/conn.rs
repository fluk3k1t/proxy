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
use rustls_platform_verifier::{BuilderVerifierExt, Verifier};
// use rustls_pki_types::ServerName;
// use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter},
    net::TcpListener,
    time::{error::Elapsed, timeout},
};
use tokio_rustls::{
    Connect, TlsAcceptor,
    client::TlsStream,
    rustls::{
        ClientConfig, RootCertStore, ServerConfig, ServerConnection, Stream,
        crypto::{self, aws_lc_rs::default_provider},
        pki_types::{PrivateKeyDer, ServerName, pem::PemObject},
    },
};
use tokio_rustls::{TlsConnector, rustls::pki_types::CertificateDer};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed, FramedRead};

use crate::{HttpDecoder, Packet};
pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection { stream }
    }

    pub async fn handle_connection(&mut self) -> Result<(), Error> {
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
            let host = packet.headers.get("host").unwrap();

            match &method[..] {
                "GET" => {
                    let response = reqwest::get(packet.path.unwrap()).await.unwrap();
                    let bytes = response_to_bytes(response).await.unwrap();

                    self.stream.write_all(&bytes).await.unwrap();
                }
                "CONNECT" => {
                    let mut upstream = self.connect_tls(host).await.unwrap();
                }
                _ => {}
            }
        } else {
            panic!("method does not exist!");
        }
    }

    async fn connect_tls(&mut self, host: &String) -> Result<TlsStream<TcpStream>, Error> {
        let (domain, port) = {
            let mut splitted = host.split(":");
            (
                splitted.next().unwrap().to_string(),
                splitted.next().unwrap_or("443").parse::<u16>().unwrap(),
            )
        };

        let arc_crypto_provider = Arc::new(default_provider());
        let config = ClientConfig::builder_with_provider(arc_crypto_provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_platform_verifier()
            .unwrap()
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(config));
        let client = TcpStream::connect((domain.clone(), port)).await.unwrap();

        let client = connector
            .connect(ServerName::try_from(domain).unwrap(), client)
            .await;

        client
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
