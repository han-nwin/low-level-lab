// NOTE: The goal for this lab is to show when to use async and when to use parallelism
// Use tokio to fetch multiple web pages concurrenlt (I/O wait)
// Perform word count in parrallel with rayon (heavy CPU work)
use anyhow::Context;
use rayon::prelude::*;
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    // tokio
    let mut handles = Vec::new();

    // NOTE: consume args into inter
    // since spawn will move url and if we take it from args
    // the tasks don't guarantee the lifetime aka args might end before the task
    for url in args.into_iter().skip(1) {
        let handle = tokio::spawn(async move {
            let response = reqwest::get(&url)
                .await
                .context("web page request failed")?;
            let text = response
                .text()
                .await
                .context("web page text parse failed")?;

            // NOTE: since the ? above return Result<> we have to make this return Result<>
            // anyhow will help with bundling different Error from different sources
            // (reqwest::Error, io::Error, etc)
            Ok::<_, anyhow::Error>((url, text))
        });
        handles.push(handle);
    }

    let mut page_texts = HashMap::new();
    for handle in handles {
        let (url, page_text) = handle.await.context("web page request failed")??;
        page_texts.insert(url, page_text);
    }

    // rayon
    page_texts.par_iter().for_each(|(url, page_text)| {
        let words: Vec<&str> = page_text.split_whitespace().collect();
        println!("{}: {}", url, words.len());
    });

    Ok(())
}
