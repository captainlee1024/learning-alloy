use alloy::{primitives::U256, providers::ProviderBuilder, sol};
use eyre::{Result, eyre};

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Counter,
    "src/artifacts/Counter.json"
);

#[tokio::main]
async fn main() -> Result<()> {
    let provider = ProviderBuilder::new().on_anvil_with_wallet();

    let contract_instance = Counter::deploy(&provider).await?;

    let call_builder = contract_instance.setNumber(U256::from(42));
    let _set_number_receipt = call_builder
        .send()
        .await
        .map_or_else(
            |e| Err(eyre!("send counter setNumber tx failed, {e:?}")),
            |pending_tx_builder| {
                println!(
                    "Pending Tx to SetNumber, tx hash: {:?}",
                    pending_tx_builder.tx_hash()
                );
                Ok(pending_tx_builder)
            },
        )?
        .get_receipt()
        .await
        .map_or_else(
            |e| Err(eyre!("Get tx receipt failed, err: {e:?}")),
            |tx_receipt| {
                println!("Get tx receipt that setNumber, receipt:\n {tx_receipt:#?}");
                Ok(tx_receipt)
            },
        )?;

    let inc_call_builder = contract_instance.increment();
    let _inc_receipt = inc_call_builder
        .send()
        .await
        .map_or_else(
            |e| Err(eyre!("Senf Counter increment tx failed, {e:?}")),
            |pending_tx_builder| {
                println!(
                    "Pending tx to increment, tx hash: {:?}",
                    pending_tx_builder.tx_hash()
                );
                Ok(pending_tx_builder)
            },
        )?
        .get_receipt()
        .await
        .map_or_else(
            |e| Err(eyre!("Get tx receipt that increment failed, {e:?}")),
            |tx_receipt| {
                println!("Get tx receipt that increment, receipt:\n {tx_receipt:#?}");
                Ok(tx_receipt)
            },
        )?;

    // 注意：由于 'solc' 生成的工件不包含命名的返回值，因此无法从工件中派生返回值名称 'number'。这意味着
    // 返回值必须由索引访问 - 就像它是一个未命名的值一样。 如果你更喜欢使用命名返回值，建议将 Solidity 代码
    // 直接嵌入到 'sol！' 宏中，如 'deploy_from_contract.rs' 所示。
    // 之后就可以这样call().await?.number; 来访问返回值了
    let get_number_call_builder = contract_instance.number();
    let number = get_number_call_builder.call().await?._0;
    println!("Get number: {number}");

    Ok(())
}
