//curl -v --http1.1 \
// -X POST \
// 'http://127.0.0.1:9999/users?id=42&active=true' \
// -H 'Accept: application/json' \
// -H 'Content-Type: application/json' \
// -H 'Connection: close' \
// --data '{"name":"Han","role":"admin"}'
use http_parser::Request;
use std::{
    io::{BufReader, prelude::*},
    net::TcpListener,
};

fn main() {
    // 1. bind a port (7878)
    let listener = TcpListener::bind("127.0.0.1:9999").unwrap_or_else(|e| {
        println!("failed to listen on port: {}", e);
        std::process::exit(1);
    });

    // 2. Read the incoming -> an iterator of type TcpStream
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        // get peer addr and local addr
        let peer_addr = stream.peer_addr().unwrap();
        let local_addr = stream.local_addr().unwrap();
        println!("new connection from: {peer_addr} to {local_addr}");

        // 3. Read each request as Buffer
        let mut buffer: [u8; 4096] = [0; 4096]; // buffer of 0s

        let bytes_read = stream.read(&mut buffer).unwrap();
        println!("Read: {bytes_read} bytes");
        // compare with wc -c (count bytes)
        // printf 'GET / HTTP/1.1\r\nHost: 127.0.0.1:9999\r\nUser-Agent: curl/8.17.0\r\nAccept: */*\r\n\r\n' | wc -c

        let parsed_request = Request::parse(&buffer);
    }
}
