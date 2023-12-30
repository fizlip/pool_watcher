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

mod abis {
    ethers::contract::abigen!(
        UniswapV3Pool,
        "/home/filip/Dokument/MEV/pool_watcher/src/uni_v3_pool.json",
        event_derives (serde::Deserialize, serde::Serialize);
    );
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
    on_swap_event(binding.0).await;
    Ok(())
}

async fn on_swap_event(sqrt_price_x96: U256) {
    let p = sqrt_price_x96.as_u128() as f64;
    let p = p * p;
    let d = 2_f64.powf(192.0);
    let p = p / d * 10_f64.powf(12.0);
    info!("{{ price: {}}}", p);

}
