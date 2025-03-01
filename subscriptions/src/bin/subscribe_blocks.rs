use alloy::{
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::BlockTransactionsKind,
    transports::http::reqwest::Url,
};
use eyre::Result;
use futures_util::{StreamExt, stream};

#[tokio::main]
async fn main() -> Result<()> {
    let main_net_ws_url = "wss://ethereum-rpc.publicnode.com";
    let main_net_ws = WsConnect::new(main_net_ws_url);
    let ws_provider = ProviderBuilder::new().on_ws(main_net_ws).await?;

    let sub = ws_provider.subscribe_blocks().await?;
    let mut sub_stream = sub.into_stream().take(5);
    // receive new header
    println!("通过websocket 订阅，实现获取最新区块, 这里展示订阅5个区块后退出");
    println!("WebSocket start");
    while let Some(new_header) = sub_stream.next().await {
        println!(
            "subscribe new block header: block number: {}, block hash: {}",
            new_header.number, new_header.hash
        );
    }
    println!("WebSocket end");

    let main_net_http_url_str = "https://ethereum-rpc.publicnode.com";
    let main_net_http_url = Url::try_from(main_net_http_url_str)?;
    let http_provider = ProviderBuilder::new().on_http(main_net_http_url);
    // start polling
    let poller = http_provider.watch_blocks().await?;
    let mut stream = poller.into_stream().flat_map(stream::iter).take(5);
    println!("通过http 轮训，实现获取最新区块, 这里展示轮训到5个区块后退出");
    println!("http poller start");
    while let Some(block_hash) = stream.next().await {
        let new_block = http_provider
            .get_block_by_hash(block_hash, BlockTransactionsKind::Full)
            .await?
            .unwrap();
        println!(
            "polling new block: block number: {}, block hash: {}",
            new_block.header.number, block_hash
        );
    }
    println!("http poller end");

    Ok(())
}
