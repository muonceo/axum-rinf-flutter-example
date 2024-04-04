use models::Counter;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

// When running on mobile, you must specify the IP address.
// static HOST: &str = "http://<api-server_ip_addr>:3000";
static HOST: &str = "http://localhost:3000";

pub async fn get_counter() -> Result<Counter> {
    // `reqwest` supports all platforms, including web.
    let counter = reqwest::get(format!("{HOST}/counter"))
        .await?
        .json::<Counter>()
        .await?;
    Ok(counter)
}

pub async fn set_counter(counter: &Counter) -> Result<bool> {
    let response = reqwest::Client::new()
        .put(format!("{HOST}/counter"))
        .json(&counter)
        .send()
        .await?;
    Ok(response.status().is_success())
}
