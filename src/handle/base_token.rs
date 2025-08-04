use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::{Add, Mul}};

pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub const WSOL: &str = "So11111111111111111111111111111111111111112";

pub const PYTH_SOL_USD_PRICE_FEED_ID: &str = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";

#[derive(Serialize, Deserialize, Debug)]
struct PythPrice {
    price: String
}

#[derive(Serialize, Deserialize, Debug)]
struct PythPricePrice {
    price: PythPrice,
}

#[derive(Serialize, Deserialize, Debug)]
struct PythParsedPrice {
    parsed: Vec<PythPricePrice>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JupiterPriceData {
    pub usd_price: f64,
}

type JupiterPriceResponse = HashMap<String, JupiterPriceData>;

#[derive(Debug, Clone, PartialEq)]
pub struct DecimalAmount {
    pub amount_in_int: u64,
    pub decimals: u8,
}

impl DecimalAmount {
    pub fn new(amount_in_int: u64, decimals: u8) -> Self {
        DecimalAmount {
            amount_in_int,
            decimals,
        }
    }

    pub fn to_float(&self) -> f64 {
        self.amount_in_int as f64 / 10f64.powi(self.decimals as i32)
    }
    
}

impl Mul for DecimalAmount {
    type Output = f64;

    fn mul(self, rhs: DecimalAmount) -> Self::Output {
        self.to_float() * rhs.to_float()
    }
}

impl Add<u64> for DecimalAmount {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        let new_amount = self.amount_in_int + rhs;
        DecimalAmount {
            amount_in_int: new_amount,
            decimals: self.decimals,
        }
    }
}

pub async fn retry_get_jupiter_price(
    mint: &str,
    url: &str,
    times: u32,
) -> Option<f64> {
    let mut attempts = 0;
    while attempts < times {
        if let Some(res) = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .map_err(|e| eprintln!("Error fetching price: {}", e))
        .ok() {
            if res.status() == reqwest::StatusCode::OK {
                let jupiter_price = res.json::<JupiterPriceResponse>().await;
                #[cfg(test)]
                println!("Jupiter price response: {:?}", jupiter_price);

                match jupiter_price {
                    Ok(price) => {
                        if let Some(parsed_price) = price.get(mint) {
                            return Some(parsed_price.usd_price);
                        }
                    },
                    Err(_) => {
                        return None
                    }
                }
            } else {
                return None
            }
        }
        attempts += 1;
    }
    None
}

pub async fn retry(
    url: &str,
    times: u32,
) -> Option<DecimalAmount> {
    let mut attempts = 0;
    while attempts < times {
        if let Some(res) = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .map_err(|e| eprintln!("Error fetching price: {}", e))
        .ok() {
            if res.status() == reqwest::StatusCode::OK {
                let pyth_price = res.json::<PythParsedPrice>().await;
                match pyth_price {
                    Ok(price) => {
                        if let Some(parsed_price) = price.parsed.first() {
                            if let Ok(price_in_int) = parsed_price.price.price.parse::<u64>() {
                                return Some(DecimalAmount::new(price_in_int, 8)); // Assuming 6 decimals for Pyth prices
                            }
                        }
                    },
                    Err(_) => {
                        return None
                    }
                }
            } else {
                return None
            }
        }
        attempts += 1;
    }
    None
}

pub async fn get_price(
    mint: &str,
    timestamp: u64,
) -> Option<DecimalAmount> {
    
    #[cfg(test)]
    println!("Fetching price for mint: {}", mint);

    match mint {
        USDC => {
            Some(DecimalAmount::new(1_000_000, 6)) // Assuming 6 decimals for USDC
        },
        USDT => {
            Some(DecimalAmount::new(1_000_000, 6)) // Assuming 6 decimals for USDT
        },
        WSOL => {
            let url = format!("https://hermes.pyth.network/v2/updates/price/{}?ids%5B%5D={}", timestamp, PYTH_SOL_USD_PRICE_FEED_ID);
            retry(&url, 3).await
        },
        _ => {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_price() {
        let price = get_price(USDC, 0).await;
        assert!(price.is_some());
        assert_eq!(price.unwrap().to_float(), 1.0);
        
        let price = get_price(WSOL, 1717532000).await;
        assert!(price.is_some());
    }
}

#[cfg(test)]
mod decimal_amount_tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_retry_get_jupiter_price() {
        let url = "https://lite-api.jup.ag/price/v3?ids=KMNo3nJsBXfcpJTVhZcXLW7RmTwTt4GVFE7suUBo9sS";
        let price = retry_get_jupiter_price("KMNo3nJsBXfcpJTVhZcXLW7RmTwTt4GVFE7suUBo9sS", url, 3).await;
        println!("Jupiter price for KMNo3nJsBXfcpJTVhZcXLW7RmTwTt4GVFE7suUBo9sS: {:?}", price);
    }
}