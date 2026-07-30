mod receiver;
mod sender;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Start the server on a separate thread
    let (server_handle, rx) = receiver::start_server();

    // Wait for the server to be ready
    let mut ready = rx.recv().unwrap();
    while ready == 0 {
        ready = rx.recv().unwrap();
    }

    // === Sender == send data forever in a separate thread
    let _client_handle = std::thread::spawn(move || {
        // NOTE: Every thread need a new tokio runtime to run async fn
        let tokio_run_time = tokio::runtime::Runtime::new().unwrap();

        tokio_run_time.block_on(async {
            loop {
                // sender send some data
                let response = sender::send_request().await;
                match response {
                    Ok(response) => {
                        println!("MAIN: {:?}", response);
                        let text = response.text().await;
                        match text {
                            Ok(text) => {
                                println!("MAIN: text: {:?}", text);
                            }
                            Err(e) => {
                                println!("MAIN: Text Error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("MAIN: Request Error: {}", e);
                        break;
                    }
                }

                // sleep a bit here
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }

            Ok::<(), anyhow::Error>(())
        })
    });

    // wait for the server thread to join back but it will never
    // aka main will wait forever here
    server_handle.join().unwrap();

    Ok(())
}
