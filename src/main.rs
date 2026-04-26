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
use proxy::Connection;
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

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let mut listener = TcpListener::bind("127.0.0.1:4443").await?;

//     // let cert_file = fs::read_to_string("ore_ca.cert").unwrap();
//     // let private_key_file = fs::read_to_string("ore_ca.key").unwrap();

//     let certs = CertificateDer::pem_file_iter("ore_ca.cert")
//         .unwrap()
//         .map(|cert| cert.unwrap())
//         .collect();

//     let private_key = PrivateKeyDer::from_pem_file("ore_ca.key").unwrap();
//     let config = ServerConfig::builder()
//         .with_no_client_auth()
//         .with_single_cert(certs, private_key)
//         .unwrap();

//     let accepter = TlsAcceptor::from(Arc::new(config));

//     loop {
//         match listener.accept().await {
//             Ok((stream, addr)) => {
//                 let mut stream = accepter.accept(stream).await;

//                 match stream {
//                     Ok(mut stream) => {
//                         let mut buf = [0; 1024];
//                         stream.read(&mut buf).await.unwrap();

//                         println!("{:?}", String::from_utf8_lossy(&buf));
//                     }
//                     Err(e) => eprintln!("{}", e),
//                 }
//                 // let mut conn = ServerConnection::new(Arc::new(config)).unwrap();

//                 // let mut tls_stream = Stream::new(&mut conn, &mut stream);
//             }
//             Err(e) => println!("couldn't get client: {:?}", e),
//         }
//     }

//     Ok(())
// }
