use alloy::{
    primitives::{U256, address},
    providers::{Provider, ProviderBuilder},
};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // create provider
    // http url
    let http_url = "https://ethereum-rpc.publicnode.com".parse()?;
    // http provider
    let http_provider = ProviderBuilder::new().on_http(http_url);

    // Get storage slot 0 from the Uniswap V3 USDC-ETH pool on Ethereum mainnet.
    let pool_address = address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");
    let storage_slot = U256::from(0);

    // 默认在latest block上调用, .block() 可以设置一个区块
    let storage = http_provider
        .get_storage_at(pool_address, storage_slot)
        .await?;
    println!("contract uniswap V3 USDC=ETH pool Sloat 0: {storage:?}");

    Ok(())
}
