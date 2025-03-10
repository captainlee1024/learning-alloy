use alloy::{
    primitives::address,
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::Filter,
};
use eyre::Result;
use futures_util::stream::{self, StreamExt};

#[tokio::main]
async fn main() -> Result<()> {
    // 两种都可以，一般推荐http链接, 监听也推荐http轮训，当前不推荐subscribe
    // subscribe支持gRPC之后再推荐使用 subscribe的方式
    // create a provider
    // http provider
    // http url
    let http_rpc_url = "https://ethereum-rpc.publicnode.com".parse()?;
    let http_provider = ProviderBuilder::new().on_http(http_rpc_url);

    // ws provider
    // ws url
    let ws_url = "wss://ethereum-rpc.publicnode.com";
    let ws_connect = WsConnect::new(ws_url);
    let ws_provider = ProviderBuilder::new().on_ws(ws_connect).await?;

    // 查询指定区块区间的所有event 数据
    // get latest block number
    let latest_block_num = http_provider.get_block_number().await?;
    // get logs from latest_block_num to latest_block_num
    let filter1 = Filter::new().from_block(latest_block_num);
    // let logs = http_provider.get_logs(&filter1).await?;
    let logs = ws_provider.get_logs(&filter1).await?;
    println!(
        "get logs from block {}, to block {}",
        latest_block_num, latest_block_num
    );

    // for log in logs {
    //     println!("log: {:?}", log);
    // }

    // 查询到的logs太多, 我们这里转换成stream打印前10个
    let mut logs_stream = stream::iter(logs).take(10);
    while let Some(log) = logs_stream.next().await {
        println!("log: {:?}", log);
    }

    // get all logs from the latest block that match the event "Transfer(address,address,uint256)"
    let filter2 = Filter::new()
        .from_block(latest_block_num)
        .event("Transfer(address,address,uint256)");
    // let transfer_logs = http_provider.get_logs(&filter2).await?;
    let transfer_logs = ws_provider.get_logs(&filter2).await?;
    println!(
        "get transfer event logs from block {}, to block {}",
        latest_block_num, latest_block_num
    );
    // for log in transfer_logs {
    //     println!("log: {:?}", log);
    // }

    // 查询到的logs太多，这里转成stream打印前10个
    let mut transfer_logs_stream = stream::iter(transfer_logs).take(10);
    while let Some(log) = transfer_logs_stream.next().await {
        println!("log: {:?}", log);
    }

    // 查询指定合约的log
    let uniswap_token_address = address!("1f9840a85d5aF5bf1D1762F925BDADdC4201F984");
    let uniswap_log_filter = Filter::new()
        .address(uniswap_token_address)
        .from_block(latest_block_num);
    let uniswap_logs = http_provider.get_logs(&uniswap_log_filter).await?;
    // let uniswap_logs = ws_provider.get_logs(&uniswap_log_filter).await?;
    println!(
        "get uniswap event logs from block {}, to block {}",
        latest_block_num, latest_block_num
    );

    for log in uniswap_logs {
        println!("log: {:?}", log);
    }

    Ok(())
}
