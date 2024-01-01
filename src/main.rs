use std::sync::Arc;
use ethers::{
  types::{U256, H160},
  providers::{Provider, Ws},
  prelude::MiddlewareBuilder,
  signers::{LocalWallet, Signer},

};

use hex_literal::hex;
use tracing::info;
use clap::Parser;

use reqwest::Error;
use serde::Deserialize;

mod abis {
    ethers::contract::abigen!(
        UniswapV3Pool,
        "/home/filip/Dokument/MEV/pool_watcher/src/uni_v3_pool.json",
        event_derives (serde::Deserialize, serde::Serialize);
    );
}

#[derive(Deserialize, Debug)]
struct Price {
    amount: String,
    base: String,
    currency: String,
}

#[derive(Deserialize, Debug)]
struct WPrice {
    data: Price
}

const UNI_V3_POOL_ADDR: H160 = H160(hex!("FAD57d2039C21811C8F2B5D5B65308aa99D31559"));

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    pub wss: String,
    #[arg(long)]
    pub private_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let ws = Ws::connect(args.wss).await.unwrap();
    let provider = Provider::new(ws);

    let wallet: LocalWallet = args.private_key.parse().unwrap();
    let address = wallet.address();
    let provider = Arc::new(provider.nonce_manager(address).with_signer(wallet.clone()));

    let univ3pool = abis::UniswapV3Pool::new(UNI_V3_POOL_ADDR, provider.clone());
    let binding = univ3pool.slot_0().call().await?;

    let univ3_link_price = on_swap_event(binding.0).await?;
    let cb_link_price = get_cb_price().await?;

    info!("{{UNI V3: {}}}", univ3_link_price);
    info!("{{CB LINK price: {}}}", cb_link_price);

    Ok(())
}

async fn get_cb_price() -> Result<String, Error> {
    let request_url = format!("https://api.coinbase.com/v2/prices/LINK-USD/spot");

    let response = reqwest::get(&request_url).await?;

    let price: WPrice = response.json().await?;
    Ok(price.data.amount)
}

async fn on_swap_event(sqrt_price_x96: U256) -> Result<f64, Error>{
    let p = sqrt_price_x96.as_u128() as f64;
    let p = p * p;
    let d = 2_f64.powf(192.0);
    let p = p / d * 10_f64.powf(12.0);
    Ok(p)

}
