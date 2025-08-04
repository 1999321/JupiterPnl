use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::option_serializer::OptionSerializer;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UserBalanceInfo {
    pub user: Pubkey,
    pub balance: u64,
    pub mint: Pubkey,
    pub decimals: u8,
}

pub fn parse_balance(
    user: &Pubkey,
    meta: &solana_transaction_status::UiTransactionStatusMeta,
) -> Vec<UserBalanceInfo> {
    let mut user_balances: Vec<UserBalanceInfo> = vec![];
    
    match meta.post_token_balances.as_ref() {
        OptionSerializer::Some(post_balances) => {
            for balance in post_balances {
                if let OptionSerializer::Some(this_user) = balance.owner.as_ref() {
                    if Pubkey::from_str_const(this_user) == *user {
                        user_balances.push(
                            UserBalanceInfo {
                                user: Pubkey::from_str_const(&this_user),
                                balance: balance.ui_token_amount.amount.parse::<u64>().unwrap_or(0),
                                mint: Pubkey::from_str_const(&balance.mint),
                                decimals: balance.ui_token_amount.decimals,
                            }
                        );
                    }
                }
            }
        },
        _ => {
            println!("No post token balances found for user {}", user);
        }
    }

    user_balances
}