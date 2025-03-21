//! Example of subscribing and listening for pending transactions in the public mempool by
//! `WebSocket` subscription.

use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use eyre::Result;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    // Create the provider.
    // let rpc_url = "wss://eth-mainnet.g.alchemy.com/v2/your-api-key";
    let rpc_url = "wss://ethereum-sepolia-rpc.publicnode.com";
    let ws = WsConnect::new(rpc_url);
    let provider = ProviderBuilder::new().on_ws(ws).await?;

    // Subscribe to pending transactions.
    // Alteratively use `subscribe_full_pending_transactions` to get the full transaction details
    // directly if supported by the RPC provider.
    // let sub = provider.subscribe_pending_transactions().await?;
    // 访问的这个节点支持 full pending tx
    // 所以可以调用该方法，来发送 "eth_subscribe", ("newPendingTransactions", true) 请求
    let sub = provider.subscribe_full_pending_transactions().await?;

    // Wait and take the next 3 transactions.
    // let mut stream = sub.into_stream().take(3);
    let mut stream = sub.into_stream().take(2);

    println!("Awaiting full pending transactions...");

    // Take the stream and print the pending transaction.
    let handle = tokio::spawn(async move {
        while let Some(tx) = stream.next().await {
            println!("Full pending transaction details: {tx:#?}");
        }
    });

    handle.await?;

    Ok(())
}
