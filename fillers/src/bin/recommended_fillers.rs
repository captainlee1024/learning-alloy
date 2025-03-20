use alloy::{
    consensus::Transaction,
    network::TransactionBuilder,
    primitives::{U256, address},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 启动anvil并创建链接
    // 在 'alloy 0.11' 之后，使用 'ProviderBuilder::new（）' 构建提供程序时，默认启用推荐的填充物
    // recommended fillers。
    // recommended fillers 默认开启了这几个
    // JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>;
    let provider = ProviderBuilder::new().on_anvil_with_wallet();

    let vitalik = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");

    // 构造tx
    // 默认开启的fillers 会进行一些缺省填充
    // 构造EIP-1599 类型的tx
    // NonceFiller 会设置nonce
    // GasFiller 会设置gas
    // ChainIdFiller 会设置chain_id
    let tx = TransactionRequest::default()
        .with_to(vitalik)
        .with_value(U256::from(100));

    // 发送交易
    let pending_tx_builder = provider.send_transaction(tx.clone()).await?;

    let tx_hash = *pending_tx_builder.tx_hash();

    let pending_tx = provider
        .get_transaction_by_hash(tx_hash)
        .await?
        .expect("Pending tx not found");

    assert_eq!(pending_tx.nonce(), 0);
    println!("Transaction sent with nonce: {}", pending_tx.nonce());
    println!(
        "该交易中recommended 设置的一些数据: nonce, gas, chain_id等, 此时发送的交易的nonce: {}, gas_limit: {}, chain id: {}",
        pending_tx.nonce(),
        pending_tx.gas_limit(),
        pending_tx.chain_id().unwrap()
    );

    let pending_tx_builder_2 = provider.send_transaction(tx).await?;
    let tx_hash_2 = *pending_tx_builder_2.tx_hash();
    let pending_tx_2 = provider
        .get_transaction_by_hash(tx_hash_2)
        .await?
        .expect("");
    assert_eq!(pending_tx_2.nonce(), 1);
    println!("Transaction sent with nonce: {}", pending_tx_2.nonce());
    println!(
        "该交易中recommended 设置的一些数据: nonce, gas, chain_id等, 此时发送的交易的nonce: {}, gas_limit: {}, chain id: {}",
        pending_tx_2.nonce(),
        pending_tx_2.gas_limit(),
        pending_tx_2.chain_id().unwrap()
    );

    Ok(())
}
