use reqwest::{Error, Response};

pub async fn send_request() -> Result<Response, Error> {
    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:8080/")
        .body("Yoooo! What's up")
        .send()
        .await?;

    Ok(response)
}
