use std::{fs, time::Duration};

use bytes::BytesMut;
use rcgen::{
    CertificateParams, CertifiedKey, Issuer, KeyPair, SigningKey, generate_simple_self_signed,
};
use tokio::{
    io::{AsyncReadExt, BufWriter},
    net::{TcpListener, TcpStream},
    time::{error::Elapsed, timeout},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = TcpListener::bind("127.0.0.1:8000").await?;

    match listener.accept().await {
        Ok((socket, addr)) => {
            tokio::spawn(handle_connection(socket));
        }
        Err(e) => println!("couldn't get client: {:?}", e),
    }

    Ok(())
}

pub struct Connection {
    // stream: BufWriter<TcpStream>,
    stream: TcpStream,
    // buffer: BytesM
    buffer: BytesMut,
}

impl Connection {
    pub async fn read_http_with_timeout(&mut self, duration: Duration) {
        // match timeout(
        //     Duration::from_millis(1000),
        //     self.stream.read(&mut self.buffer),
        // )
        // .await?
        // {
        //     Ok(n) => {}
        //     Err(e) => {}
        // }
    }

    pub async fn read_http(&mut self) {
        loop {
            // self.stream.read_buf(&mut self.buffer).await;
            // self.stream.t
            // self.stream.readable().await;
            // self.stream.try_read_buf()
        }
    }
}

async fn handle_connection(mut stream: TcpStream) -> Result<(), Elapsed> {
    let mut tmp = [0; 1024];

    loop {}
}
