use reqwest::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    #[serde(rename = "retCode")]
    pub ret_code: i32,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    pub result: T,
}

#[derive(Debug, Deserialize)]
pub struct TickerResult {
    pub list: Vec<Ticker>,
}

#[derive(Debug, Deserialize)]
pub struct KlineResult {
    pub list: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Ticker {
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    #[serde(rename = "highPrice24h")]
    pub high_price_24h: String,
    #[serde(rename = "lowPrice24h")]
    pub low_price_24h: String,
    #[serde(rename = "volume24h")]
    pub volume_24h: String,
    #[serde(rename = "turnover24h")]
    pub turnover_24h: String,
}

pub struct BybitClient {
    client: reqwest::Client,
    base_url: String,
}

impl BybitClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.bybit.com".to_string(),
        }
    }

    pub async fn get_spot_ticker(&self, symbol: &str) -> Result<Option<Ticker>, Error> {
        let url = format!("{}/v5/market/tickers", self.base_url);
        
        let response = self.client
            .get(&url)
            .query(&[
                ("category", "spot"),
                ("symbol", symbol)
            ])
            .send()
            .await?;

        if response.status().is_success() {
            let api_response: ApiResponse<TickerResult> = response.json().await?;
            
            if api_response.ret_code == 0 {
                Ok(api_response.result.list.into_iter().next())
            } else {
                eprintln!("API Error: {}", api_response.ret_msg);
                Ok(None)
            }
        } else {
            response.error_for_status()?;
            Ok(None)
        }
    }

    pub async fn get_tickers(&self, category: &str) -> Result<Vec<Ticker>, Error> {
        let url = format!("{}/v5/market/tickers", self.base_url);
        
        let response = self.client
            .get(&url)
            .query(&[
                ("category", category),
            ])
            .send()
            .await?;

        if response.status().is_success() {
            let api_response: ApiResponse<TickerResult> = response.json().await?;
            
            if api_response.ret_code == 0 {
                Ok(api_response.result.list)
            } else {
                eprintln!("API Error: {}", api_response.ret_msg);
                Ok(Vec::new())
            }
        } else {
            response.error_for_status()?;
            Ok(Vec::new())
        }
    }

    pub async fn get_daily_kline(&self, symbol: &str, limit: usize) -> Result<Vec<Vec<String>>, Error> {
        let url = format!("{}/v5/market/kline", self.base_url);
        
        let response = self.client
            .get(&url)
            .query(&[
                ("category", "spot"),
                ("symbol", symbol),
                ("interval", "D"),
                ("limit", &limit.to_string()),
            ])
            .send()
            .await?;

        if response.status().is_success() {
            let api_response: ApiResponse<KlineResult> = response.json().await?;
            
            if api_response.ret_code == 0 {
                Ok(api_response.result.list)
            } else {
                eprintln!("API Error fetching kline for {}: {}", symbol, api_response.ret_msg);
                Ok(Vec::new())
            }
        } else {
            response.error_for_status()?;
            Ok(Vec::new())
        }
    }
}
