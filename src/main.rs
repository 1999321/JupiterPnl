#[warn(deprecated)]
pub mod tx;
pub mod handle;
pub mod utils;

use std::str::FromStr;

use axum::{
    routing::get,
    http::StatusCode,
    Json, Router,
    extract::Query
};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::handle::handle_swap_item::Pnl;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UserInfo {
    pub user_address: String,
    pub token_mint: String,
}

#[tokio::main]
async fn main() {
    
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/", get(root))
        .route("/pnl", get(get_pnl));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, Jupiter!"
}

async fn get_pnl(
    Query(user_info): Query<UserInfo>,
) -> (StatusCode, Json<Pnl>) {

    println!("User Info: {:?}", user_info);
    let jupiterv6_indexer_im = tx::jupiterv6_indexer::JupiterV6Indexer::new();
    let mint = Pubkey::from_str(&user_info.token_mint);
    let user = Pubkey::from_str(&user_info.user_address);
    if mint.is_err() || user.is_err() {
        return (StatusCode::BAD_REQUEST, Json(Pnl::default()));
    }
    let res = jupiterv6_indexer_im.get_jupiter_v6_txs(&user.unwrap(), &mint.unwrap()).await;

    (
        StatusCode::OK,
        Json(res)
    )
}