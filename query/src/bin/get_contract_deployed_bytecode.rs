use alloy::{
    primitives::address,
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

    let pool_address = address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");
    let bytecode = http_provider.get_code_at(pool_address).await?;

    println!("contract uniswap V3 USDC-ETH pool's bytecode: {bytecode:?}");

    Ok(())
}
