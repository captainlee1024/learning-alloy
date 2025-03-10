#![feature(slice_pattern)]

use alloy::{
    primitives::address,
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::{BlockNumberOrTag, Filter},
    sol,
    sol_types::SolEvent,
    transports::http::reqwest::Url,
};
use eyre::Result;
use futures_util::stream::{self, StreamExt};

sol! {
    event Transfer(address indexed from, address indexed to, uint256 amount);
}

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
        // println!("address: {:?}", log.topics()[0]);
        // topics0 不是合约地址，是event 签名
        println!("log event signathre: {:?}", log.topics()[0]);
        println!("sol gen signature hash: {}", Transfer::SIGNATURE_HASH);

        // let value = U256::from_be_slice(log.data().data.as_slice());
        // 手动解析
        // let data = log.data().data.as_slice(); // 获取 &[u8]
        // let data_bytes: [u8; 32] = data.try_into().expect("Data must be 32 bytes");
        // let value = U256::from_be_bytes(data_bytes);
        //
        // println!(
        //     "手动解析数据 --> 在 block {:?} Tx {:?} 中 {:?} 转账 {:?} USDT 到 {:?}",
        //     log.block_number,
        //     log.transaction_hash,
        //     log.topics()[1], // topics()[1]是地址补0之后的数据，下面通过sol
        //     // 生成结构体之后解析出的数据是真正的地址
        //     value,
        //     log.topics()[2]
        // );

        // 使用sol 去生成对应的合约数据类型，方便查询后的数据解码到对应的数据类型
        let Transfer { from, to, amount } = log.log_decode()?.inner.data;
        println!(
            "使用sol生成中间数据类型自动解析数据 --> 在 block {:?} Tx {:?} 中 {from} 转账给 {to} {amount} USDT",
            log.block_number, log.transaction_hash
        )
    }
    println!("websocket subscribe USDT Transfer Event end");
    println!();

    // 使用 http polling 的方式获取最新 logs
    let http_url_str = "https://ethereum-rpc.publicnode.com";
    let http_url = Url::try_from(http_url_str)?;
    let http_provider = ProviderBuilder::new().on_http(http_url);

    // watch_logs 实际是轮训 eth_getFilterChanges, 每次查询返回自上次查询后新增的logs
    // logs_poller 是一个轮训器, into_stream的到Stream<Item = Vec<Log>>
    // flat_map 是将每个Vec<Log> 转换为 Stream<Item = Log> 并展平
    let logs_poller = http_provider.watch_logs(&filter).await?;
    let mut stream = logs_poller.into_stream().flat_map(stream::iter).take(40);
    // let mut stream = logs_poller.into_stream();
    println!(
        "使用 http poll 的方式订阅新区块的logs, 并通过filter 过滤，这里获取20event后退出进行演示"
    );
    println!("http polling USDT Transfer Event start");
    while let Some(log) = stream.next().await {
        // let value = U256::from_be_slice(log.data().data.as_slice());
        //
        //       let data = log.data().data.as_slice(); // 获取 &[u8]
        //       let data_bytes: [u8; 32] = data.try_into().expect("Data must be 32 bytes");
        //       let value = U256::from_be_bytes(data_bytes);
        //
        //       println!("address: {:?}", log.topics()[0]);
        //       println!(
        //           "在 block {:?} Tx {:?} 中 {:?} 转账 {:?} USDT 到 {:?}",
        //           log.block_number,
        //           log.transaction_hash,
        //           log.topics()[1],
        //           value,
        //           log.topics()[2]
        //       );

        // 使用sol生成合约交互中对应到rust的数据类型
        let Transfer { from, to, amount } = log.log_decode()?.inner.data;
        println!(
            "block {:?} tx {:?} {from} transfer {to} {amount} USDT",
            log.block_number, log.transaction_hash
        );
    }
    println!("http polling USDT Transfer Event end");

    Ok(())
}
