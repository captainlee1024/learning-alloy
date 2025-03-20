use alloy::{
    contract::{ContractInstance, Interface},
    dyn_abi,
    primitives::U256,
    providers::ProviderBuilder,
    sol,
};
use eyre::{Result, eyre};

// setup, 这个有字节码用来部署合约
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Counter,
    "src/artifacts/Counter.json"
);

/// 不使用sol!根据合约地址和ABI创建出一个新的合约实例
/// 注意：使用这种方法和合约交互时传递的参数和返回值类型都是DynSolxxx的类型
#[tokio::main]
async fn main() -> Result<()> {
    let provider = ProviderBuilder::new().on_anvil_with_wallet();

    // 部署合约，生成的合约实例不使用
    let contract_instance = Counter::deploy(provider.clone()).await?;

    // 使用sol!根据合约地址和ABI创建出一个新的合约实例
    println!(
        "使用ContractInstance::new() 根据address, abi, provider 创建contract instance并和合约交互"
    );

    // 合约地址
    let contract_address = contract_instance.address();
    // 合约artifacts文件路径 (合约artifacts文件里包含`abi`, `bytecode`, `deployedBytecode` and `metadata`), 只有合约ABI文件也可
    // let path = std::env::current_dir()?.join("learning-alloy/contracts/src/artifacts/Counter.json");
    let path = std::env::current_dir()?.join("contracts/src/artifacts/Counter.json");

    // 读取artifact
    let artifact = std::fs::read(path).expect("Failed to read artifact");
    let json: serde_json::Value = serde_json::from_slice(&artifact)?;

    // 从artifact获取ABI 的值
    let abi_value = json.get("abi").expect("Failed to get ABI from artifact");
    // 构造abi json
    let abi = serde_json::from_str(&abi_value.to_string())?;

    // 根据 json abi, contract address, provider 创建Contract Instance
    let contract_instance_without_sol =
        ContractInstance::new(*contract_address, &provider, Interface::new(abi));

    // 不实用宏不能构造出CounterInstance, 只能构造出ContractInstance
    // let call_builder = contract_instance_with_abi.setNumber(U256::from(42));
    let call_builder = contract_instance_without_sol
        .function("setNumber", &[dyn_abi::DynSolValue::from(U256::from(42))])?;

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
                println!("Get tx receipt that setNumber, receipt:\n {tx_receipt:?}");
                Ok(tx_receipt)
            },
        )?;

    // let inc_call_builder = contract_instance_without_sol.increment();
    let inc_call_builder = contract_instance_without_sol.function("increment", &[])?;
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
                println!("Get tx receipt that increment, receipt:\n {tx_receipt:?}");
                Ok(tx_receipt)
            },
        )?;

    // 注意：由于 'solc' 生成的工件不包含命名的返回值，因此无法从工件中派生返回值名称 'number'。这意味着
    // 返回值必须由索引访问 - 就像它是一个未命名的值一样。 如果你更喜欢使用命名返回值，建议将 Solidity 代码
    // 直接嵌入到 'sol！' 宏中，如 'deploy_from_contract.rs' 所示。
    // 之后就可以这样call().await?.number; 来访问返回值了
    // let get_number_call_builder = contract_instance.number();
    let get_number_call_builder = contract_instance_without_sol.function("number", &[])?;
    let number_value = get_number_call_builder.call().await?;
    let number = number_value.first().unwrap().as_uint().unwrap().0;

    println!("Get number: {number}");

    // 调用一个不存在的方法
    let unknown_function = contract_instance_without_sol
        .function("decrement", &[])
        .unwrap_err();
    assert!(
        unknown_function
            .to_string()
            .contains("function decrement does not exist")
    );

    Ok(())
}
