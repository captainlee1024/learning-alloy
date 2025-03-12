use alloy::{
    primitives::U256,
    providers::{Provider, ProviderBuilder},
    sol,
};
use eyre::{Result, eyre};

sol! {
    #[allow(missing_docs)]
    // solc v0.8.26; solc Counter.sol --viz-ir --optimize --bin
    #[sol(rpc, bytecode="6080806040523460135760df908160198239f35b600080fdfe6080806040526004361015601257600080fd5b60003560e01c9081633fb5c1cb1460925781638381f58a146079575063d09de08a14603c57600080fd5b3460745760003660031901126074576000546000198114605e57600101600055005b634e487b7160e01b600052601160045260246000fd5b600080fd5b3460745760003660031901126074576020906000548152f35b34607457602036600319011260745760043560005500fea2646970667358221220e978270883b7baed10810c4079c941512e93a7ba1cd1108c781d4bc738d9090564736f6c634300081a0033")]
    contract Counter {
        uint256 public number;
        function setNumber(uint256 newNumber) public {
            number = newNumber;
        }

        function increment() public {
            number++;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Spin up anvil node
    // create provider
    let provider = ProviderBuilder::new().on_anvil_with_wallet();

    // deploy contract
    let contract = Counter::deploy(&provider).await?;
    println!("Deployed contract as address: {:?}", contract.address());

    // Set number to 42
    let call_builder = contract.setNumber(U256::from(42));
    let pending_transaction_builder = call_builder.send().await?;
    // wait for tx confirm
    // pending_transaction_builder.get_receipt() wait for tx confirm, and then fetch receipt
    let tx_hash = pending_transaction_builder.watch().await?;
    println!("Set number to 42 at tx: {tx_hash}");
    // we can also fetch the receipt by using the tx hash
    // 这里查询的时候返回的是Result<Option<t>>的类型
    // 所以可以.map_err()?.inspect()调用，因为?解构之后还有一层Option
    // 可以调用inspect
    let _receipt = provider
        .get_transaction_receipt(tx_hash)
        .await
        .map_err(|e| eyre!("fetch tx: {tx_hash:?} receipt failed, {e:?}"))?
        .inspect(|tx_receipt| println!("tx already confirmed, tx receipt: {:#?}", tx_receipt));

    // 这里在PendingTxBuilder上调用get_receipt返回的是Result<t>
    // 所以不能map_err()?.inspect
    // 使用map_or_else代替，传递两个闭包进去做err的转换和日志的打印
    // Increment the number to 43
    contract
        .increment()
        .send()
        .await
        .map_or_else(
            |e| Err(eyre!("send increment tx failed, {e:?}")),
            |pending_tx| {
                println!("pending increment tx, tx hash: {:?}", pending_tx.tx_hash());
                Ok(pending_tx)
            },
        )?
        .get_receipt()
        .await
        .map_or_else(
            |e| Err(eyre!("get tx receipt failed, {e:?}")),
            |tx_receipt| {
                println!("tx already confirmed, receipt: {tx_receipt:#?}");
                Ok(tx_receipt)
            },
        )?;

    // get the number which should be 43
    let get_number_call_builder = contract.number();
    let number = get_number_call_builder.call().await?.number;
    println!("get numner: {number:?}");

    Ok(())
}
