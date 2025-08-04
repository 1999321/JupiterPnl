use std::collections::HashMap;

use solana_sdk::{pubkey::Pubkey, signature::Signature};

use crate::handle::base_token::{self, DecimalAmount};

#[derive(Debug, Clone)]
pub struct SwapSumInfos {
    pub swap_data: Vec<crate::tx::inner_tx::SwapInstruction>,
    pub token_data: Vec<crate::tx::post_balance::UserBalanceInfo>,
    pub timestamp: u64,
    pub sig: Signature,
}

impl SwapSumInfos {
    pub fn new(
        swap_data: Vec<crate::tx::inner_tx::SwapInstruction>,
        token_data: Vec<crate::tx::post_balance::UserBalanceInfo>,
        timestamp: u64,
        sig: Signature,
    ) -> Self {
        SwapSumInfos {
            swap_data,
            token_data,
            timestamp,
            sig,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwapItem {
    pub sig: Signature,
    pub timestamp: u64,
    pub mint: Pubkey,
    pub amount: f64,
    pub usd_value: f64,
    pub buy_amount: f64, // Amount bought in USD
    pub sell_amount: f64, // Amount sold in USD
    pub buy_usd_value: f64, // USD value of the amount bought
    pub sell_usd_value: f64, // USD value of the amount sold
}

impl SwapItem {
    pub async fn new(
        mint: Pubkey,
        swap_sum_infos: SwapSumInfos
    ) -> Self {

        #[cfg(test)]
        println!("Handle Tx: {}", swap_sum_infos.sig);

        let mut token_decimals = HashMap::new();
        let mut token_prices = HashMap::new();
        swap_sum_infos.token_data.iter().for_each(|a|{
            token_decimals.insert(a.mint, a.decimals);
        });

        let mut amount = 0.0;
        let mut usd_value:f64 = 0.0;
        let mut buy_amount = 0.0;
        let mut sell_amount = 0.0;
        let mut buy_usd_value = 0.0;
        let mut sell_usd_value = 0.0;

        for a in &swap_sum_infos.swap_data {
            if a.input_mint == mint {
                let output_mint = &a.output_mint;
                let output_mint_decimals = token_decimals.get(output_mint).unwrap_or(&0);
                let input_mint_decimals = token_decimals.get(&a.input_mint).unwrap_or(&0);
                if *output_mint_decimals == 0 || *input_mint_decimals == 0 {
                    continue; // Skip if decimals are not found
                }
                let output_amount = DecimalAmount::new(
                    a.output_amount,
                    *output_mint_decimals,
                );
                let input_amount = DecimalAmount::new(
                    a.input_amount,
                    *input_mint_decimals,
                );
                if token_prices.get(output_mint).is_none() {
                    let mint_price = base_token::get_price(&output_mint.to_string(), swap_sum_infos.timestamp).await;
                    if let Some(price) = mint_price {
                        token_prices.insert(output_mint.clone(), price);
                    }
                }
                let mint_price = token_prices.get(output_mint).cloned();
                if let Some(price) = mint_price {
                    let input_amount_f64 = input_amount.to_float();
                    amount -= input_amount_f64;
                    sell_amount += input_amount_f64;

                    #[cfg(test)]
                    println!("Sell USD Value: {} {}", output_amount.to_float(), price.to_float());

                    let sell_usd_value_f64 = output_amount * price;
                    usd_value -= sell_usd_value_f64;
                    sell_usd_value += sell_usd_value_f64;
                }
            }

            if a.output_mint == mint {
                let input_mint = &a.input_mint;
                let input_mint_decimals = token_decimals.get(input_mint).unwrap_or(&0);
                let output_mint_decimals = token_decimals.get(&a.output_mint).unwrap_or(&0);
                if *input_mint_decimals == 0 || *output_mint_decimals == 0 {
                    continue; // Skip if decimals are not found
                }
                let input_amount = DecimalAmount::new(
                    a.input_amount,
                    *input_mint_decimals,
                );
                let output_amount = DecimalAmount::new(
                    a.output_amount,
                    *output_mint_decimals,
                );
                
                if token_prices.get(input_mint).is_none() {
                    let mint_price = base_token::get_price(&input_mint.to_string(), swap_sum_infos.timestamp).await;
                    if let Some(price) = mint_price {
                        token_prices.insert(input_mint.clone(), price);
                    }
                }
                let mint_price = token_prices.get(input_mint).cloned();
                if let Some(price) = mint_price {
                    let output_amount_f64 = output_amount.to_float();
                    amount += output_amount_f64;
                    buy_amount += output_amount_f64;

                    #[cfg(test)]
                    println!("Buy USD Value: {} {}", input_amount.to_float(), price.to_float());

                    let buy_usd_value_f64 = input_amount * price;
                    usd_value += buy_usd_value_f64;
                    buy_usd_value += buy_usd_value_f64;
                }
            }
        };

        SwapItem {
            sig: swap_sum_infos.sig,
            timestamp: swap_sum_infos.timestamp,
            mint: mint, // Placeholder, should be set appropriately
            amount: amount, // Placeholder, should be set appropriately
            usd_value: usd_value, // Placeholder, should be calculated based on the swap data
            buy_amount: buy_amount,
            sell_amount: sell_amount,
            buy_usd_value: buy_usd_value,
            sell_usd_value: sell_usd_value,
        }
    }
}