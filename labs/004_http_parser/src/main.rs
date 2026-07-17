//curl -v --http1.1 \
// -X POST \
// 'http://127.0.0.1:9999/users?id=42&active=true' \
// -H 'Accept: application/json' \
// -H 'Content-Type: application/json' \
// -H 'Connection: close' \
// --data '{"name":"Han","role":"admin"}'
use http_parser::Parser
use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn main() {
    // 1. bind a port (7878)
    let listener = TcpListener::bind("127.0.0.1:9999").unwrap_or_else(|e| {
        println!("failed to listen on port: {}", e);
        std::process::exit(1);
    });

    // 2. Read the incoming -> an iterator of type TcpStream
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // get peer addr and local addr
        let peer_addr = stream.peer_addr().unwrap();
        let local_addr = stream.local_addr().unwrap();
        println!("new connection from: {peer_addr} to {local_addr}");

        // 3. Read each request as Buffer
        let buf_reader = BufReader::new(&stream);

        let raw_request: Vec<_> = buf_reader.lines().map(|result| result.unwrap()).collect();
        println!("raw_request: {:?}", raw_request);

        let parsed_request = Parser::parse(raw_request);

        println!("parsed_request: {}", parsed_request);
    }
}
