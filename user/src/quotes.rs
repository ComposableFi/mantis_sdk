use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Query {
    pub src_chain: String,
    pub dst_chain: String,
    pub token_in: String,
    pub token_out: String,
    #[serde(rename = "amount_in")]
    pub amount: String,
    pub src_address: String,
    pub dst_address: String,
}

pub enum QueryError {
    HttpError(reqwest::Error),
    AuctioneerError(String),
}

impl Query {
    pub async fn exec_with(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<QuoteResponse, QueryError> {
        let request = client
            .get(url)
            .header("Content-Type", "application/json")
            .json(self);
        let response = request.send().await.map_err(|e| QueryError::HttpError(e))?;
        if response.status().is_success() {
            response.json().await.map_err(|e| QueryError::HttpError(e))
        } else {
            Err(QueryError::AuctioneerError(response.text().await.map_err(|e| QueryError::HttpError(e))?))
        }
    }

    pub async fn exec(&self, url: &str) -> Result<QuoteResponse, QueryError> {
        let client = reqwest::Client::new();
        self.exec_with(&client, url).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeanQuote {
    #[serde(rename = "token_out")]
    pub token: String,
    #[serde(rename = "amount_out")]
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Quote {
    pub solver_id: String,
    #[serde(flatten)]
    pub quote: MeanQuote,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub mean_output: MeanQuote,
    pub outputs: Vec<Quote>,
}
