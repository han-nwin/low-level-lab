// A prelude is usually a convenience module containing commonly used types,
// traits, functions, and macros.
use http_server_rust::ThreadPool;
use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn main() {
    // 1. bind a port (7878)
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap_or_else(|e| {
        println!("failed to listen on port: {}", e);
        std::process::exit(1);
    });

    // 3.1 create a new thread pool with 4 threads
    let pool = ThreadPool::new(4).expect("failed to create thread pool");

    // 2. Read the incoming -> an iterator of type TcpStream
    // NOTE: take(2) 2 connections to test
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap_or_else(|e| {
            eprintln!("failed to accept: {}", e);
            std::process::exit(1);
        });

        // get peer addr
        let peer_addr = stream.peer_addr().unwrap_or_else(|e| {
            println!(
                "failed to get peer addr: {}. Giving a dummy addr and port",
                e
            );
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 9999) // dummy addr and port
        });
        // get local addr (should be 127.0.0.1:7878)
        let local_addr = stream.local_addr().unwrap_or_else(|e| {
            eprintln!("failed to get local addr: {}", e);
            std::process::exit(1);
        });
        println!("new connection from: {peer_addr} to {local_addr}");

        // 3. Handle the connection with thread pool
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("main: Shutting down...");
}

fn handle_connection(mut stream: TcpStream) {
    // sleep 5 sec to simulate some work
    thread::sleep(Duration::from_secs(5));

    // NOTE: The BufReader adds buffering by managing calls to the std::io::Read trait methods for us.
    let buf_reader = BufReader::new(&stream);
    // let the compiler infer the type of the elements in Vec with _
    //
    // `take_while()` takes a closure as an argument. It will call this
    // closure on each element of the iterator, and yield elements
    // while it returns `true`.
    //
    // After `false` is returned, `take_while()`'s job is over, and the
    // rest of the elements are ignored.
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // println!("http_request: {:?}", http_request);

    let request_line = http_request[0].clone(); // get the first line
    // [..] create a &str view into String
    let (status_line, file_path) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "src/hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "src/hello.html")
        }
        _ => ("HTTP/1.1 404 Not Found", "src/404.html"),
    };
    // NOTE: Responses have the following format:
    // HTTP-Version Status-Code Reason-Phrase CRLF
    // headers CRLF
    // message-body
    // -----
    // HTTP/1.1 200 OK\r\n
    // Content-Length: 100\r\n
    // \r\n
    // <html>...</html>
    let contents = fs::read_to_string(file_path).unwrap_or_else(|e| {
        println!("failed to read content file: {}", e);
        std::process::exit(1);
    });
    let content_length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {content_length}\r\n\r\n {contents}"); // \r\n = CRNF: carriage return and newline

    stream.write_all(response.as_bytes()).unwrap_or_else(|e| {
        println!("failed to write response: {}", e);
        std::process::exit(1);
    });
}
