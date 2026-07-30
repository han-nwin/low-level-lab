use std::sync::mpsc;
use tokio::io::{AsyncReadExt, AsyncWriteExt}; // for reading and wrting
use tokio::runtime::Runtime;

pub fn start_server() -> (std::thread::JoinHandle<()>, mpsc::Receiver<u8>) {
    let (tx, rx) = mpsc::channel(); //communcation signal for main.rs
    tx.send(0).unwrap(); // 0 mean not ready
    // create new thread
    let handle = std::thread::spawn(move || {
        // NOTE: Every thread need a new tokio runtime to run async fn
        // create tokio runtime and execute the async server
        let run_time = Runtime::new().unwrap();
        run_time.block_on(async {
            // Bind to an address
            let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
                .await
                .unwrap();

            println!("SERVER: Server listening on 127.0.0.1:8080");

            tx.send(1).unwrap(); // 1 mean ready

            // Server running
            server(listener).await.unwrap();
        })
    });
    (handle, rx)
}

pub async fn server(listener: tokio::net::TcpListener) -> Result<(), std::io::Error> {
    // Accept connections forever
    loop {
        // waiting for a connection if get one return socket, addr tuple
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        // Handle this client in an async task
        tokio::spawn(async move {
            let mut buf = [0_u8; 1024];

            //read and write loop
            loop {
                match socket.read(&mut buf).await {
                    Ok(n) => {
                        if n == 0 {
                            break;
                        }
                        println!(
                            "SERVER: Received data:\n {}",
                            str::from_utf8(&buf[0..n]).unwrap()
                        );
                    }
                    Err(e) => {
                        println!("SERVER: Error: {}", e);
                        break;
                    }
                }

                match socket
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 16\r\n\r\nYoooo! What's up")
                    .await
                {
                    Ok(_) => {
                        println!("SERVER: Sent data");
                    }
                    Err(e) => {
                        println!("SERVER: Error: {}", e);
                        break;
                    }
                };
            }
        });
    }
}
