#![feature(slice_pattern)]

use core::slice::SlicePattern;

use alloy::{
    primitives::{U256, address},
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::{BlockNumberOrTag, Filter},
    transports::http::reqwest::Url,
};
use eyre::Result;
use futures_util::stream::{self, StreamExt};

#[tokio::main]
async fn main() -> Result<()> {
    // 使用 subscribe 的方式获取最新的logs
    let ws_url = "wss://ethereum-rpc.publicnode.com";
    let ws_connect = WsConnect::new(ws_url);
    let ws_provider = ProviderBuilder::new().on_ws(ws_connect).await?;

    let usdt_address = address!("0xdAC17F958D2ee523a2206206994597C13D831ec7");

    let filter = Filter::new()
        .address(usdt_address)
        .event("Transfer(address,address,uint256)")
        .from_block(BlockNumberOrTag::Latest);

    let sub = ws_provider.subscribe_logs(&filter).await?;
    let mut sub_stream = sub.into_stream().take(40);
    println!(
        "使用 subscribe 的方式订阅新区块的logs, 并通过filter 过滤，这里获取20个event 后退出进行演示"
    );
    println!("websocket subscribe USDT Transfer Event start");
    while let Some(log) = sub_stream.next().await {
        // let value = U256::from_be_slice(log.data().data.as_slice());
        let data = log.data().data.as_slice(); // 获取 &[u8]
        let data_bytes: [u8; 32] = data.try_into().expect("Data must be 32 bytes");
        let value = U256::from_be_bytes(data_bytes);

        println!("address: {:?}", log.topics()[0]);
        println!(
            "在 block {:?} Tx {:?} 中 {:?} 转账 {:?} USDT 到 {:?}",
            log.block_number,
            log.transaction_hash,
            log.topics()[1],
            value,
            log.topics()[2]
        );
    }
    println!("websocket subscribe USDT Transfer Event end");
    println!();

    // 使用 http polling 的方式获取最新 logs
    let http_url_str = "https://ethereum-rpc.publicnode.com";
    let http_url = Url::try_from(http_url_str)?;
    let http_provider = ProviderBuilder::new().on_http(http_url);

    let logs_poller = http_provider.watch_logs(&filter).await?;
    let mut stream = logs_poller.into_stream().flat_map(stream::iter).take(40);
    println!(
        "使用 http poll 的方式订阅新区块的logs, 并通过filter 过滤，这里获取20event后退出进行演示"
    );
    println!("http polling USDT Transfer Event start");
    while let Some(log) = stream.next().await {
        // let value = U256::from_be_slice(log.data().data.as_slice());

        let data = log.data().data.as_slice(); // 获取 &[u8]
        let data_bytes: [u8; 32] = data.try_into().expect("Data must be 32 bytes");
        let value = U256::from_be_bytes(data_bytes);

        println!("address: {:?}", log.topics()[0]);
        println!(
            "在 block {:?} Tx {:?} 中 {:?} 转账 {:?} USDT 到 {:?}",
            log.block_number,
            log.transaction_hash,
            log.topics()[1],
            value,
            log.topics()[2]
        );
    }
    println!("http polling USDT Transfer Event end");

    Ok(())
}
