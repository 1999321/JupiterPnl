use std::cmp;
use std::str::FromStr;
use std::sync::Arc;

use dashmap::DashMap;
use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_client::rpc_config::RpcTransactionConfig;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;
use tokio::task::JoinSet;
use crate::handle::handle_swap_item::Pnl;

use crate::tx::{inner_tx, post_balance};

const JUPITER_V6_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
const RPC_URL: &str = "https://fluent-compatible-choice.solana-mainnet.quiknode.pro/a0bf531d3ea905572f5d6d4f008f76e2a7ea1ced/";//"https://solana-mainnet.gateway.tatum.io";//"https://go.getblock.us/a2e90ccfc62448eeb000ac41771bf260";//"https://chain.gawallet.org/solana-mainnet/";

#[derive(Clone)]
struct CloneableRpcClient {
    inner: Arc<RpcClient>,
}

pub struct JupiterV6Indexer {
    client: CloneableRpcClient,
    _jupiter_v6_id: Pubkey,
}

impl JupiterV6Indexer {

    pub fn new() -> Self {
        let client = Arc::new(RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::finalized()));
        let _jupiter_v6_id = Pubkey::from_str(JUPITER_V6_ID).expect("Failed to parse Jupiter V6 ID");

        let client = CloneableRpcClient {
            inner: client.clone(),
        };

        JupiterV6Indexer {
            client,
            _jupiter_v6_id,
        }
    }

    pub async fn get_jupiter_v6_txs(
        &self,
        user_pubkey: &Pubkey,
        token_pubkey: &Pubkey,
    ) -> Pnl {

        let associated_token_account = self.client.inner.get_token_accounts_by_owner(user_pubkey, TokenAccountsFilter::Mint(token_pubkey.clone()));
        if associated_token_account.is_err() {
            eprintln!("Error fetching associated token account: {}", associated_token_account.err().unwrap());
            return Pnl::default();
        }
        let associated_token_account = associated_token_account;
        if associated_token_account.is_err() {
            eprintln!("No associated token account found for user: {}", user_pubkey);
            return Pnl::default();
        }
        let associated_token_account = associated_token_account.unwrap();
        if associated_token_account.is_empty() {
            eprintln!("No associated token account found for user: {}", user_pubkey);
            return Pnl::default();
        }
        let associated_token_account = &associated_token_account[0].pubkey;
        let associated_token_account = Pubkey::from_str(&associated_token_account);
        if associated_token_account.is_err() {
            return Pnl::default();
        }
        let associated_token_account = associated_token_account.unwrap();

        #[cfg(test)]
        println!("Associated Token Account: {:?}", associated_token_account);// 默认获取第一个关联代币账户

        let signatures = self.get_transaction_signatures(&associated_token_account).await;

        #[cfg(test)]
        if let Some(signatures) = signatures.clone() {
            for signature in signatures {
                println!("Transaction Signature: {}", signature);
            }
        } else {
            println!("No transactions found for the user.");
        }

        //#[cfg(not(test))]
        match signatures {
            Some(signatures) => {
                // 分组
                let mut tasks = JoinSet::new();
                let signatures = Arc::new(signatures);
                //let signatures = Arc::new(vec![Signature::from_str("4NXAjsFhCwDczensh61NQdrcVRNMdtF6g8ytkJEdNQFirALr1Fg3yFoDXqU5AyP8PhzUXSHA8BDySVevLCtCkRPj").unwrap()]);
                let txs = Arc::new(DashMap::new());

                // 同时保持最多25个连接
                for chunk in signatures.chunks(cmp::max(signatures.len() / 25, 1)) {
                    let chunk = chunk.to_vec();
                    let client = Arc::clone(&self.client.inner);
                    let txs = Arc::clone(&txs);

                    tasks.spawn(async move {
                        for sig in chunk {
                            let attemp_times = 5;
                            // 重试获取交易
                            for _ in 0..attemp_times {
                                if let Some(tx) = client.get_transaction_with_config(
                                    &sig,
                                    RpcTransactionConfig {
                                        encoding: Some(UiTransactionEncoding::JsonParsed),
                                        commitment: Some(CommitmentConfig::finalized()),
                                        max_supported_transaction_version: Some(1), // 关键：声明支持的事务版本；事务版本 0 是 Solana 当前默认的事务版本，表示使用基础的事务格式。
                                    },
                                ).ok() {
                                    txs.insert(sig, tx);

                                    #[cfg(test)]
                                    println!("Successfully fetched transaction for signature: {}", sig);

                                    break; // 成功获取交易后跳出重试循环
                                } else {

                                    #[cfg(test)]
                                    eprintln!("Failed to fetch transaction for signature: {}", sig);

                                    continue;
                                };
                            }
                        }
                    });

                }

                // 等待所有任务完成
                while let Some(res) = tasks.join_next().await {
                    match res {
                        Ok(_) => {},
                        Err(e) => eprintln!("Error in task: {}", e),
                    }
                }

                let mut swap_sum_infos = vec![];

                // 获取数据
                txs.iter().for_each(|entry| {
                    let value = entry.value();
                    let block_time = value.block_time.unwrap_or(0) as u64;
                    let sig = entry.key().clone();
                    match value.transaction.meta.clone() {
                        Some(meta) => {
                            #[cfg(test)]
                            println!("Processing transaction: {} at block time: {}", sig, block_time);

                            let swap_data = inner_tx::parse(&meta);
                            let token_data = post_balance::parse_balance(user_pubkey, &meta);
                            swap_sum_infos.push(crate::handle::handle_tx::SwapSumInfos::new(swap_data, token_data, block_time, sig));
                        },
                        None => {
                            println!("No meta data found for transaction: {}", entry.key());
                        }
                    }
                });

                let mut tasks = JoinSet::new();
                let swap_items = Arc::new(DashMap::new());

                for chunk in swap_sum_infos.chunks(cmp::max(swap_sum_infos.len() / 25, 1)) {
                    let chunk = chunk.to_vec();
                    let token_pubkey = token_pubkey.clone();
                    let swap_items = Arc::clone(&swap_items);

                    tasks.spawn(async move {
                        for swap_sum_info in chunk {
                            let sig = swap_sum_info.sig.clone();
                            let swap_item = crate::handle::handle_tx::SwapItem::new(token_pubkey, swap_sum_info).await;
                            swap_items.insert(sig, swap_item);
                        }
                    });
                }

                // 等待所有任务完成
                while let Some(res) = tasks.join_next().await {
                    match res {
                        Ok(_) => {},
                        Err(e) => eprintln!("Error in task: {}", e),
                    }
                }

                let mut sort_swap_items = swap_items.iter()
                    .map(|entry| entry.value().clone())
                    .collect::<Vec<_>>();

                // 按照 timestamp 排序
                sort_swap_items.sort_by(|a,b| a.timestamp.cmp(&b.timestamp));

                let pnl = Pnl::new(token_pubkey.clone(), sort_swap_items).await;
                return pnl;

            },
            None => {
                println!("No signatures found for user {}", user_pubkey);
                return Pnl::default();
            }
        }

    }

    pub async fn get_transaction_signatures(
        &self,
        user_pubkey: &Pubkey,
    ) -> Option<Vec<Signature>> {
        // 由于速率限制等因素，避免过长无反应，最多查找100个
        let config = GetConfirmedSignaturesForAddress2Config {
            limit: Some(25),// 指定返回的交易签名的最大数量，None表示不限制
            before: None,// 指定一个交易签名，返回的结果将是该签名之前的交易签名
            until: None,// 指定一个交易签名，返回的结果将是该签名之后的交易签名
            commitment: Some(CommitmentConfig::finalized()),// 指定查询的确认级别
            //min_context_slot: None,// 区域为指定slot到目前slot，None表示不限制
        };

        let signatures = self.client.inner.get_signatures_for_address_with_config(user_pubkey, config);

        match signatures {
            Ok(signatures) => {
                Some(signatures.into_iter().map(|s| Signature::from_str(&s.signature).unwrap_or(Signature::default())).collect())
            },
            Err(e) => {
                eprintln!("Error fetching signatures: {}", e);
                None
            }
        }
    }
} 

// test
#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_jupiter_v6_txs() {
        let indexer = JupiterV6Indexer::new();//DMoie6GXkodFQYp2MDf5eBj48GR9F9AWHBnT9jZ7g1zC
        let user_pubkey = Pubkey::from_str("J14Cg556roeBSgWFEKNTiSQeydMPRW6FZNB2zDMmSadQ").expect("Failed to parse user public key");
        let token_pubkey = Pubkey::from_str("KMNo3nJsBXfcpJTVhZcXLW7RmTwTt4GVFE7suUBo9sS").expect("Failed to parse user public key");

        let res = indexer.get_jupiter_v6_txs(&user_pubkey, &token_pubkey).await;
        println!("Result: {:?}", res);
    }
}
