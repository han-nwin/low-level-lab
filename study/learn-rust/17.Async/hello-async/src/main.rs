use std::future::Future;
use trpl::{Either, Html};

//NOTE:
// async fn  is the same as fn -> Future<Output = >

// fn page_title(url: &str) -> impl Future<Output = Option<String>> {
async fn page_title(url: &str) -> (&str, Option<String>) {
    // let response = trpl::get(url).await;
    // let response_text = response.text().await;
    let response_text = trpl::get(url).await.text().await;
    let title = Html::parse(&response_text)
        .select_first("title")
        .map(|title| title.inner_html());
    (url, title)
}

// main can't be async, we need an async runtime instead
// Most languages that support async bundle a runtime, but
// Rust does not. Instead, there are many different
// async runtimes available, each of which makes different tradeoffs
// suitable to the use case it targets. For example,
// a high-throughput web server with many CPU cores and a large amount
// of RAM has very different needs than a microcontroller with a single core,
// a small amount of RAM, and no heap allocation ability. The crates that
// provide those runtimes also often supply async versions of common
// functionality such as file or network I/O.
fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Behind the scenes, calling block_on sets up a runtime using the tokio crate
    trpl::block_on(async {
        let future_1 = page_title(&args[1]);
        let future_2 = page_title(&args[2]);

        let (url, maybe_title) = match trpl::select(future_1, future_2).await {
            Either::Left(left) => left,
            Either::Right(right) => right,
        };

        println!("{url} return first");

        match maybe_title {
            Some(title) => println!("It's title is {title}"),
            None => println!("It has no title"),
        }
    });
}
//cargo run -- "https://www.rust-lang.org"
