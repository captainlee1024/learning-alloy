use alloy::{
    consensus::Transaction,
    network::{Ethereum, EthereumWallet, TransactionBuilder},
    node_bindings::Anvil,
    primitives::{U256, address},
    providers::{
        PendingTransactionBuilder, Provider, ProviderBuilder,
        fillers::{
            BlobGasFiller, ChainIdFiller, GasFiller, JoinFill, NonceFiller, SimpleNonceManager,
        },
    },
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use eyre::{Ok, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 启动anvil
    let anvil = Anvil::new().block_time(1).try_spawn()?;
    // 使用anvil 的内置账户创建钱包
    let pk: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::new(pk);

    // =========================不使用任何filler
    // 此时nonce, chain_id, gas_limit, max_fee_per_gas都需要手动添加
    // 这些都是一个交易的必须项
    // 创建provider时不启用recommended fillres
    let provider_without_fillers = ProviderBuilder::default()
        // add the WalletFiller to the provider
        .wallet(wallet.clone())
        .on_http(anvil.endpoint_url());

    // 此时没有开启任何fillers, 需要手动填充nonce等
    let vitalik = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
    let tx = TransactionRequest::default()
        .with_to(vitalik)
        .with_value(U256::from(100))
        .with_nonce(0)
        .with_chain_id(provider_without_fillers.get_chain_id().await?)
        .with_gas_limit(300000)
        .with_max_fee_per_gas(20_000_000_000u128)
        .with_max_priority_fee_per_gas(1_000_000_000u128);
    let pending_tx_builder_1: PendingTransactionBuilder<Ethereum> = provider_without_fillers
        .send_transaction(tx.clone())
        .await?;

    let tx_hash_1 = *pending_tx_builder_1.tx_hash();
    let pending_tx_1 = provider_without_fillers
        .get_transaction_by_hash(tx_hash_1)
        .await?
        .expect("Pending tx not found");
    println!("\n手动管理填充 Nonce, ChainId, Gas, BlobGas 的交易数据:");
    println!(
        "Transaction sent with nonce: {}, gas: {}, chain_id: {}, max_fee_per_gas: {}, max_priority_fee_per_gas: {:#?}",
        pending_tx_1.nonce(),
        pending_tx_1.gas_limit(),
        pending_tx_1.chain_id().unwrap(),
        pending_tx_1.max_fee_per_gas(),
        pending_tx_1.max_priority_fee_per_gas()
    );

    // =========================使用filler
    // 使用nonce filler, gas filler, chani id filler来进行自动填充
    // 这里ProviderBuilder<L,F,N>推导不出N的类型(有两个 Ethereun和AnyNetwork)，需要手动指定
    let provider_with_fillers = ProviderBuilder::<_, _, Ethereum>::default()
        .filler(
            JoinFill::<NonceFiller<SimpleNonceManager>, ChainIdFiller>::new(
                // 自动管理填充Nonce
                // nonce manager 有两个
                // CachedNonceManager 会在本地维护，从本地获取
                // SimpleNonceManager 每次都从RPC节点获取, 会多一次网络通信开销
                NonceFiller::new(SimpleNonceManager::default()),
                // 自动填充chain_id
                ChainIdFiller::new(anvil.chain_id().into()),
            ), // .join_with::<GasFiller>(GasFiller::default())
               // .join_with(BlobGasFiller::default()),
        )
        // 自动填充gas
        .filler(GasFiller::default())
        // 自动填充BlobGas
        .filler(BlobGasFiller::default())
        // Add the WalletFiller to the Provider
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    let tx_2 = TransactionRequest::default()
        .with_to(vitalik)
        .with_value(U256::from(100));
    let pendign_tx_builder_2 = provider_with_fillers.send_transaction(tx_2.clone()).await?;
    let tx_hash_2 = *pendign_tx_builder_2.tx_hash();

    let pending_tx_2 = provider_with_fillers
        .get_transaction_by_hash(tx_hash_2)
        .await?
        .expect("Pending tx2 not found");
    println!("\n使用Nonce, ChainId, Gas, BlobGas Fillers的交易数据:");
    println!(
        "Transaction sent with nonce: {}, gas: {}, chain_id: {}, max_fee_per_gas: {}, max_priority_fee_per_gas: {:#?}",
        pending_tx_2.nonce(),
        pending_tx_2.gas_limit(),
        pending_tx_2.chain_id().unwrap(),
        pending_tx_2.max_fee_per_gas(),
        pending_tx_2.max_priority_fee_per_gas()
    );

    Ok(())
}
