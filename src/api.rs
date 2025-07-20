use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TickerPrice {
    pub symbol: String,
    pub price: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub volume: String,
    pub high_24h: String,
    pub low_24h: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KlineData {
    pub open_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub close_time: i64,
}

pub struct BinanceApi {
    base_url: String,
    client: reqwest::Client,
}

impl BinanceApi {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_ticker_prices(&self, symbols: &[String]) -> Result<HashMap<String, TickerPrice>> {
        let mut prices = HashMap::new();
        
        for symbol in symbols {
            let url = format!("{}/api/v3/ticker/24hr?symbol={}", self.base_url, symbol);
            let response = self.client.get(&url).send().await?;
            
            if response.status().is_success() {
                let ticker: serde_json::Value = response.json().await?;
                
                let price = TickerPrice {
                    symbol: symbol.clone(),
                    price: ticker["lastPrice"].as_str().unwrap_or("0").to_string(),
                    price_change: ticker["priceChange"].as_str().unwrap_or("0").to_string(),
                    price_change_percent: ticker["priceChangePercent"].as_str().unwrap_or("0").to_string(),
                    volume: ticker["volume"].as_str().unwrap_or("0").to_string(),
                    high_24h: ticker["highPrice"].as_str().unwrap_or("0").to_string(),
                    low_24h: ticker["lowPrice"].as_str().unwrap_or("0").to_string(),
                };
                
                prices.insert(symbol.clone(), price);
            }
        }
        
        Ok(prices)
    }

    pub async fn get_klines(&self, symbol: &str, interval: &str, limit: u32) -> Result<Vec<KlineData>> {
        let url = format!(
            "{}/api/v3/klines?symbol={}&interval={}&limit={}",
            self.base_url, symbol, interval, limit
        );
        
        let response = self.client.get(&url).send().await?;
        let klines: Vec<Vec<serde_json::Value>> = response.json().await?;
        
        let mut result = Vec::new();
        for kline in klines {
            if kline.len() >= 7 {
                result.push(KlineData {
                    open_time: kline[0].as_i64().unwrap_or(0),
                    open: kline[1].as_str().unwrap_or("0").to_string(),
                    high: kline[2].as_str().unwrap_or("0").to_string(),
                    low: kline[3].as_str().unwrap_or("0").to_string(),
                    close: kline[4].as_str().unwrap_or("0").to_string(),
                    volume: kline[5].as_str().unwrap_or("0").to_string(),
                    close_time: kline[6].as_i64().unwrap_or(0),
                });
            }
        }
        
        Ok(result)
    }
} 