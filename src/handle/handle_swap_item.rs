use crate::{handle::{base_token, handle_tx}, utils::f64_tool::{f64_keep_two, f64_to_percentage}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pnl {
    pub average_cost: Option<f64>,
    pub profit_loss_percentage: Option<String>,
    pub profit_loss_value: Option<f64>,
    pub unrealized_profit_loss_value: Option<f64>,
}

impl Pnl {
    pub async fn new(
        mint: solana_sdk::pubkey::Pubkey,
        swap_items: Vec<handle_tx::SwapItem>,
    ) -> Self {

        #[cfg(test)]
        for item in &swap_items {
            println!("Swap Item: {:?}", item);
        }

        let mut pnl = Pnl {
            average_cost: None,
            profit_loss_percentage: None,
            profit_loss_value: None,
            unrealized_profit_loss_value: None,
        };

        let mut sum_amount = 0.0;
        let mut sum_buy_amount = 0.0;
        let mut sum_buy_usd_value = 0.0;
        let mut sum_sell_amount = 0.0;
        let mut sum_sell_usd_value = 0.0;

        swap_items.iter().for_each(|item| {
            sum_amount += item.buy_amount;
            sum_buy_usd_value += item.buy_usd_value;
            sum_buy_amount += item.buy_amount;

            // 由于RPC的获取范围，所以只统计该范围的买入卖出情况
            match item.sell_amount == 0.0 {
                false => {
                    if item.sell_amount > sum_amount {
                        sum_sell_amount += sum_amount;
                        sum_sell_usd_value += sum_amount * item.sell_usd_value / item.sell_amount;
                        sum_amount = 0.0;
                    } else {
                        sum_sell_amount += item.sell_amount;
                        sum_sell_usd_value += item.sell_usd_value;
                        sum_amount -= item.sell_amount;
                    }
                },
                _ => {
                    
                }
            }

        });

        if sum_buy_amount > 0.0 {
            pnl.average_cost = Some(sum_buy_usd_value / sum_buy_amount);
        }

        let current_price = base_token::retry_get_jupiter_price(
            &mint.to_string(), 
            &format!(
                "https://lite-api.jup.ag/price/v3?ids={}",
                mint.to_string()
            ), 
            5
        ).await;

        if sum_sell_amount > 0.0 && sum_buy_amount > 0.0 {
            pnl.profit_loss_value = Some((sum_sell_usd_value / sum_sell_amount - sum_buy_usd_value / sum_buy_amount) * sum_sell_amount);
            pnl.unrealized_profit_loss_value = if let Some(price) = current_price {
                Some(sum_amount * (price - sum_buy_usd_value / sum_buy_amount))
            } else {
                None
            };
            pnl.profit_loss_percentage = if let Some(profit) = pnl.profit_loss_value {
                if sum_buy_usd_value > 0.0 {
                    Some(f64_to_percentage(profit / sum_buy_usd_value * 100.0))
                } else {
                    None
                }
            } else {
                None
            };
        }

        // 保留两位小数
        if let Some(avg_cost) = pnl.average_cost {
            pnl.average_cost = Some(f64_keep_two(avg_cost));
        }

        if let Some(pl_value) = pnl.profit_loss_value {
            pnl.profit_loss_value = Some(f64_keep_two(pl_value));
        }

        if let Some(unrealized_pl_value) = pnl.unrealized_profit_loss_value {
            pnl.unrealized_profit_loss_value = Some(f64_keep_two(unrealized_pl_value));
        }

        pnl
    }
}

impl Default for Pnl {
    fn default() -> Self {
        Pnl {
            average_cost: None,
            profit_loss_percentage: None,
            profit_loss_value: None,
            unrealized_profit_loss_value: None,
        }
    }
}

