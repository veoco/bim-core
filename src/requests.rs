use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use bytes::{BufMut, BytesMut};
use rand::prelude::*;
use reqwest::{header, Body, Url};
use tokio::net::TcpStream;
use tokio::sync::{watch::Receiver, Barrier, Mutex};

pub async fn request_tcp_ping(host: &SocketAddr) -> Result<u128, Box<dyn Error + Send + Sync>> {
    let now = Instant::now();
    let _stream = TcpStream::connect(host).await?;
    let used = now.elapsed().as_micros();
    Ok(used)
}

pub async fn request_http_download(
    mut url: Url,
    addr: SocketAddr,
    barrier: Arc<Barrier>,
    stop_rx: Receiver<&str>,
    counter: Arc<Mutex<u128>>,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let u = url.clone();
    let domain = u.host_str().unwrap();

    let u = url.clone();
    let q = u.query();

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("bim 1"),
    );

    let client = reqwest::Client::builder()
        .resolve(domain, addr)
        .default_headers(headers)
        .timeout(Duration::from_secs(15))
        .build()?;

    let _r = barrier.wait().await;
    while *stop_rx.borrow() != "stop" {
        let r = random::<f64>().to_string();
        let rq = format!("r={}", r);
        let query = match q {
            Some(s) => rq +"&" + s,
            None => rq,
        };
        url.set_query(Some(&query));

        let mut stream = match client.get(url.clone()).send().await {
            Ok(s) => s,
            Err(_) => continue,
        };
        while let Some(chunk) = stream.chunk().await? {
            let mut count = counter.lock().await;
            *count += chunk.len() as u128;
            if *stop_rx.borrow() == "stop" {
                break;
            }
        }
    }

    Ok(true)
}

pub async fn request_http_upload(
    mut url: Url,
    addr: SocketAddr,
    barrier: Arc<Barrier>,
    stop_rx: Receiver<&str>,
    counter: Arc<Mutex<u128>>,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let s = "0123456789AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz-=".repeat(512);

    let u = url.clone();
    let domain = u.host_str().unwrap();

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("bim 1"),
    );

    let _r = barrier.wait().await;
    while *stop_rx.borrow() != "stop" {
        let mut data = BytesMut::new();
        data.put(s.as_bytes());

        let c = counter.clone();
        let s = async_stream::stream! {
            loop {

                let chunk: Result<BytesMut, std::io::Error> = Ok(data.clone());
                let mut count = c.lock().await;
                *count += 32768;
                yield chunk;
            }
        };

        let body = Body::wrap_stream(s);
        let client = reqwest::Client::builder()
            .resolve(domain, addr)
            .default_headers(headers.clone())
            .build()?;
        
        let r = random::<f64>().to_string();
        let query = format!("r={}", r);
        url.set_query(Some(&query));

        let _res = client
            .post(url.clone())
            .body(body)
            .timeout(Duration::from_secs(15))
            .send()
            .await;
    }

    Ok(true)
}
