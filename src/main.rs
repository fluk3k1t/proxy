// use std::{
// io::{BufRead, BufReader, Read, Write},
// net::{TcpListener, TcpStream},
// };

use dns_lookup::lookup_host;
use httparse::{EMPTY_HEADER, Request};
use std::collections::HashMap;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = TcpListener::bind("127.0.0.1:8000").await?;

    // for stream in listener.incoming() {
    //     match stream {
    //         Ok(stream) => {
    //             handle_connection(stream).await;
    //         }
    //         Err(e) => { /* connection failed */ }
    //     }
    // }
    match listener.accept().await {
        Ok((socket, addr)) => {
            handle_connection(socket).await;
        }
        Err(e) => println!("couldn't get client: {:?}", e),
    }

    Ok(())
}

async fn handle_connection(mut stream: TcpStream) {
    // let mut reader = BufReader::new(stream);

    let mut tmp = [0; 1024];
    let mut header = Vec::new();

    loop {
        // let n = stream.read(&mut tmp).unwrap();
        let n = stream.try_read(&mut tmp).unwrap();

        header.extend_from_slice(&tmp[..n]);
        if tmp.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }

    // let header = String::from_utf8_lossy(&header);

    let mut headers = [EMPTY_HEADER; 64];
    let mut req = Request::new(&mut headers);

    req.parse(&header).unwrap();

    let method = req.method.unwrap();
    let path = req.path.unwrap();

    // println!("{}", method);
    // println!("{:?}", req.headers);

    let host = req
        .headers
        .iter()
        .find(|h| h.name == "Host")
        .map(|h| String::from_utf8_lossy(h.value))
        .expect("'Host' header is required");

    match method {
        "GET" => {
            // println!("{:?}", path);
            // println!("{:?}", req.headers);

            let resp = reqwest::get(format!("http://{}", host)).await.unwrap();

            println!("{:?}", resp.headers());

            let mut presp = format!("HTTP/1.1 {}\n", resp.status());
            for (n, v) in resp.headers() {
                let v = String::from_utf8_lossy(v.as_bytes());
                presp.push_str(&format!("{}: {}\r\n", n, v));
            }
            presp.push_str("\r\n");

            let headers = resp.headers().clone();

            let body = resp.text().await.unwrap();

            if headers.iter().any(|(h, v)| h == "Transfer-Encoding") {
                presp.push_str(&format!("{:x}\r\n{}", body.len(), body));
            } else {
                presp.push_str(body.as_str());
            }

            stream.write_all(presp.as_bytes()).await.unwrap();
            // stream.write_all(src)
        }
        "CONNECT" => {
            let mut s = host.split(":");
            let domain = s.next().expect("domain is required");
            let port = s.next().unwrap_or("443").parse().unwrap();

            let ip = lookup_host(domain)
                .expect(&format!("can't resolve '{}'", domain))
                .next()
                .unwrap();

            // let addr = Socket

            let mut pclinet = TcpStream::connect((ip, port)).await.unwrap();

            // println!("{:?}", pclinet);
            // let mut tmp = [0; 1024];
            let mut tmp = [0; 10240];
            // let mut header = Vec::new();

            let connect_accept = "HTTP/1.1 200 Connection Established\r\n\r\n";
            stream.write_all(connect_accept.as_bytes()).await.unwrap();

            let mut tmp1 = [0; 1024];
            let mut tmp2 = [0; 1024];

            // let mut s1 = stream
            let (mut r1, mut w1) = stream.into_split();

            let (mut r2, mut w2) = pclinet.into_split();

            let j = tokio::spawn(async move {
                loop {
                    r1.ready(Interest::READABLE).await.unwrap();

                    let n = r1.read(&mut tmp1).await.unwrap();
                    // r1.read
                    let tmp1 = &tmp1[..n];

                    w2.write_all(&tmp1).await.unwrap();
                }
            });

            tokio::spawn(async move {
                loop {
                    r2.ready(Interest::READABLE).await.unwrap();
                    let n = r2.read(&mut tmp2).await.unwrap();
                    let tmp2 = &tmp2[..n];

                    w1.write_all(&tmp2).await.unwrap();
                }
            });

            j.await.unwrap();

            // loop {
            // let n = stream.read(&mut tmp).unwrap();

            // let tmp = &tmp[..n];
            // // header.extend_from_slice(&tmp[..n]);

            // let re = tls_parser::parse_tls_plaintext(&tmp).unwrap();
            // println!("{:?}", re.1);
            // println!("{:?}", String::from_utf8_lossy(&header));
            // if tmp.windows(4).any(|w| w == b"\r\n\r\n") {
            //     break;
            // }
            // }
            // std::io::copy(&mut stream, &mut pclinet).unwrap();
            // std::io::copy(&mut pclinet, &mut stream).unwrap();

            // println!("{:?}", header);
        }
        _ => unimplemented!(),
    }
}
