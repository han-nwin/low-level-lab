// NOTE: Throughout the chapter, we’ve used the Future, Stream, and StreamExt traits in various ways.
// We need to understand them more, along with Pin and Unpin

fn main() {
    // NOTE: Rust Future trait looks like this
    use std::pin::Pin;
    use std::task::{Context, Poll};

    pub trait Future {
        type Output; // what the future resolves to.

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
    }

    // The Poll type
    pub enum Poll<T> {
        Ready(T),
        Pending,
    }
    // This Poll type is similar to an Option.
    // It has one variant that has a value,
    // Ready(T), and one that does not
    // , Pending. Poll means something quite different from
    // Option, though! The Pending variant indicates that
    // the future still has work to do, so
    // the caller will need to check again later.
    // The Ready variant indicates that the Future has finished
    // its work and the T value is available.
    //

    //NOTE: When you see code that uses await,
    //Rust compiles it under the hood to code that calls poll
    let page_title_fut = page_title(url).poll(); // this return a Pool
    loop {
        match page_title_fut {
            Ready(page_title) => match page_title {
                Some(title) => println!("The title is {title}"),
                None => println!("No title found"),
            },
            Pending => {
                //continue looping
            }
        }
    }

    //NOTE:
    //If Rust compiled it to exactly that code, though,
    //every await would be blocking—exactly the opposite of what
    //we were going for! Instead, Rust ensures that the
    //loop can hand off control to something that can pause work
    //on this future to work on other futures and then check
    //this one again later. As we’ve seen,
    //that something is an async runtime, and this scheduling and
    //coordination work is one of its main jobs.
}
