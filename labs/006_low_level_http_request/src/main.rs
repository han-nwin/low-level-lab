// use std::io::{Read, Write};
// use std::net::{TcpStream, ToSocketAddrs};
//
// tokio way
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs, lookup_host};
use tokio::runtime::Builder;

#[derive(Debug)]
struct Request {}

#[derive(Debug)]
struct Response {}

#[derive(Debug)]
enum RequestError {
    Socket,
    SocketIterator,
    TcpConnect,
}

// NOTE: DOING IT THE std::net way -> not async
// fn get(url: &str) -> Result<Response, RequestError> {
//     // turn the url string to a socket addrs
//     // make it to iterator -> call next()
//     let url_socket_addr = url
//         .to_socket_addrs()
//         .map_err(|_| RequestError::Socket)?
//         .next()
//         .ok_or(RequestError::SocketIterator)?;
//     // we can skip this step but wanna do it to show how it works
//
//     println!("url_socket_addr: {url_socket_addr:?}");
//
//     // connect to the socket addrs
//     let mut stream = TcpStream::connect(url_socket_addr).map_err(|_| RequestError::TcpConnect)?;
//
//     let request_bytes = b"GET / HTTP/1.1\r\nHost: www.google.com\r\nConnection: close\r\n\r\n";
//
//     stream
//         .write_all(request_bytes)
//         .map_err(|_| RequestError::Socket)?;
//     let mut buffer = Vec::new();
//     stream
//         .read_to_end(&mut buffer)
//         .map_err(|_| RequestError::Socket)?;
//
//     let response_text = str::from_utf8(&buffer).map_err(|_| RequestError::Socket)?;
//     let first_100_words = response_text
//         .split_whitespace()
//         .take(100)
//         .collect::<Vec<_>>()
//         .join(" ");
//     println!("first_100_words: {first_100_words}");
//
//     Ok(Response {})
// }

// NOTE: Doing it the tokio async way
async fn get(url: &str) -> Result<Response, RequestError> {
    // we can skip this step, but this is the way tokio provide async dns look up
    // return iterator -> call next() then open it with ok_or
    let url_socket_addr = lookup_host(url)
        .await
        .map_err(|_| RequestError::Socket)?
        .next()
        .ok_or(RequestError::SocketIterator)?;

    // async connect
    let mut stream = TcpStream::connect(url_socket_addr)
        .await
        .map_err(|_| RequestError::TcpConnect)?;

    let request_bytes = b"GET / HTTP/1.1\r\nHost: www.google.com\r\nConnection: close\r\n\r\n";

    stream
        .write_all(request_bytes)
        .await
        .map_err(|_| RequestError::Socket)?;

    let mut buffer = Vec::new();
    stream
        .read_to_end(&mut buffer)
        .await
        .map_err(|_| RequestError::Socket)?;

    let response_text = str::from_utf8(&buffer).map_err(|_| RequestError::Socket)?;
    let first_100_words = response_text
        .split_whitespace()
        .take(100)
        .collect::<Vec<_>>()
        .join(" ");
    println!("first_100_words: {first_100_words}");

    Ok(Response {})
}

fn main() {
    // normal std::io way
    // match get("www.google.com:80") {
    //     Ok(response) => println!("response: {response:?}"),
    //     Err(error) => println!("error: {error:?}"),
    // };

    // tokio way
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("thread-one")
        .thread_stack_size(3 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap_or_else(|e| panic!("thread pool error: {}", e));

    runtime.block_on(async {
        match get("www.google.com:80").await {
            Ok(response) => println!("response: {response:?}"),
            Err(error) => println!("error: {error:?}"),
        };
    });
}
