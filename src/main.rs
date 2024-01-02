use std::sync::Arc;
use ethers::middleware::Middleware;

use ethers::{
  types::{U256, H160},
  providers::{Provider, Ws},
  prelude::{MiddlewareBuilder, SignerMiddleware},
  signers::{LocalWallet, Signer},
};

use std::fs::File;
use std::time;
use std::thread;
use std::io::prelude::*;

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
    #[arg(long)]
    pub time: i32
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut t = 0;
    
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let time:i32 = args.time; // Experiment time

    let ws = Ws::connect(args.wss).await.unwrap();
    let provider = Provider::new(ws);

    let wallet: LocalWallet = args.private_key.parse().unwrap();
    let address = wallet.address();
    //let provider = Arc::new(provider.nonce_manager(address).with_signer(wallet.clone()));

    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    println!(
        "💫 Welcome to the Pool Watcher💫\n We will use wallet with address: {}",
        wallet.address()
    );


    while t < time {
        let univ3pool = abis::UniswapV3Pool::new(UNI_V3_POOL_ADDR, provider.clone().into());
        let binding = univ3pool.slot_0().call().await?;

        let univ3_link_price = on_swap_event(binding.0).await?;
        let cb_link_price = get_cb_price().await?;

        let spread:f64 = univ3_link_price - cb_link_price;

        if(spread > 0.03) {
            let balance_from = provider.get_balance("0xeBec795c9c8bBD61FFc14A6662944748F299cAcf", None).await?;

            info!("{{T: {}, UNI: {}, CB: {}, Spread: {}, B: {}}}", 
                  t, 
                  univ3_link_price, 
                  cb_link_price,
                  spread,
                  balance_from
                );
            // Sell UNIV3 BUY CB
        }
        if(spread < -0.03) {
            info!("{{T: {}, UNI: {}, CB: {}, Spread: {}}}", 
                  t, 
                  univ3_link_price, 
                  cb_link_price,
                  spread
                );
            // Sell UNIV3 BUY CB
        }
        
        let thousand_millis = time::Duration::from_millis(1000);
        thread::sleep(thousand_millis);
        t += 1;
    }

    Ok(())
}

//async fn write_to_file(val: String) -> Result<_ ,Error>{
//    if t == 0 {
//        let mut file = File::create("arb-result.csv")?;
//        let line = format!("{},{}\n", 
//                            t, 
//                            spread.abs()
//                            );
//        file.write_all(line.as_bytes())?;
//    }
//
//    else {
//        let mut f = File::options().append(true).open("arb-result.csv")?;
//        let spread:f64 = univ3_link_price - cb_link_price;
//        writeln!(&mut f, "{}", format!("{},{}", 
//                                        t, 
//                                        spread.abs()
//                                        ))?;
//    }
//}

async fn get_cb_price() -> Result<f64, Error> {
    let request_url = format!("https://api.coinbase.com/v2/prices/LINK-USD/spot");

    let response = reqwest::get(&request_url).await?;

    let price: WPrice = response.json().await?;
    Ok(price.data.amount.parse().unwrap())
}

async fn on_swap_event(sqrt_price_x96: U256) -> Result<f64, Error>{
    let p = sqrt_price_x96.as_u128() as f64;
    let p = p * p;
    let d = 2_f64.powf(192.0);
    let p = p / d * 10_f64.powf(12.0);
    Ok(p)

}
