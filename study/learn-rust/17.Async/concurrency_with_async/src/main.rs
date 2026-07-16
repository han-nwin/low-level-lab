use std::time::Duration;

fn main() {
    trpl::block_on(async {
        trpl::spawn_task(async {
            for i in 1..5 {
                println!("hi number {i} from the first task");
                trpl::sleep(Duration::from_millis(500)).await;
            }
        }); // task 1

        for i in 1..10 {
            println!("hi number {i} from the second task");
            trpl::sleep(Duration::from_millis(500)).await;
        }

        // === a different way to do it, no need spawn_task
        println!("=== Write differently ===");
        let fut1 = async {
            for i in 1..10 {
                println!("hi number {i} from the first task!");
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        let fut2 = async {
            for i in 1..5 {
                println!("hi number {i} from the second task!");
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        trpl::join(fut1, fut2).await; //trpl::join function is fair, 
        // we would see the order of message more "fair"
        // === //

        // == SEND DATA between 2 Async Tasks==//
        println!("SEND DATA b/w Futures (Async Tasks)");
        let (tx, mut rx) = trpl::channel();

        // To get the behavior we want, where the sleep delay happens
        // between each message, we need to put the tx and rx operations in their
        // own async blocks, as shown in Listing 17-11. Then the runtime can execute each of them separately using trpl::join,
        // put tx and rx inside a future (tx_fut, rx_fut)
        let tx_fut = async move {
            // 'move' tx inside here so it'll close after being out of scope
            let vals = vec![
                String::from("hi"),
                String::from("from"),
                String::from("the"),
                String::from("future"),
            ];

            for val in vals {
                tx.send(val).unwrap_or_else(|e| eprintln!("Error: {:?}", e));
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        let rx_fut = async {
            while let Some(message) = rx.recv().await {
                println!("received '{message}'");
            }
        };

        trpl::join(tx_fut, rx_fut).await; // join tx rx future here

        // == multi channel
        println!("Multi channels");
        let (tx, mut rx) = trpl::channel();
        let tx1 = tx.clone();

        let tx_fut = async move {
            let vals = vec![
                String::from("hi"),
                String::from("from"),
                String::from("the"),
                String::from("future"),
            ];

            for val in vals {
                tx.send(val).unwrap_or_else(|e| eprintln!("Error: {:?}", e));
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        let tx1_fut = async move {
            let vals = vec![
                String::from("more"),
                String::from("message"),
                String::from("for"),
                String::from("you"),
            ];

            for val in vals {
                tx1.send(val)
                    .unwrap_or_else(|e| eprintln!("Error: {:?}", e));
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        let rx_fut = async {
            while let Some(message) = rx.recv().await {
                println!("received '{message}'");
            }
        };

        // multi channle join! macro
        trpl::join!(tx_fut, tx1_fut, rx_fut);
    });
}
