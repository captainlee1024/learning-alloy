use alloy::{primitives::U256, providers::ProviderBuilder, sol};

use crate::Errors::{ErrorsErrors, SomeCustomError};
use eyre::Result;

// Define a custom error using the sol! macro.
sol! {
    // solc: 0.8.25; solc DecodingRevert.sol --optimize --bin
    #[allow(missing_docs)]
    #[derive(Debug, PartialEq, Eq)]
    library Errors {
        error SomeCustomError(uint256 a);
        error AnotherError(uint64 b);
    }

        #[derive(Debug)]
    #[sol(rpc, bytecode = "6080604052348015600e575f80fd5b5060a780601a5f395ff3fe6080604052348015600e575f80fd5b50600436106026575f3560e01c8063b48fb6cf14602a575b5f80fd5b60396035366004605b565b603b565b005b60405163810f002360e01b81526004810182905260240160405180910390fd5b5f60208284031215606a575f80fd5b503591905056fea26469706673582212200898a6b7d5b1bcc62a40abf2470704fe9c6cd850c77b0654134fc0ecbf0d5e6f64736f6c63430008190033")]
    contract ThrowsError {
        function error(uint256 a) external {
           revert Errors.SomeCustomError(a);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup an Anvil provider with a wallet.
    // Make sure `anvil` is in your $PATH.
    let provider = ProviderBuilder::new().on_anvil_with_wallet();

    // Deploy the contract.
    let contract = ThrowsError::deploy(&provider).await?;

    // Call the `error` function which will revert with a custom error.
    let err = contract.error(U256::from(1)).call().await.unwrap_err();

    println!("\n=== 新增 as_revert_data()方法");
    let err_revert_data = err.as_revert_data().unwrap();
    println!("err revert data: {:?}", err_revert_data);

    println!("\n=== 新增 as_decoded_error()方法");
    println!("知道revert的是哪个自定义error的话可以直接反序列化到对应的结构体");
    let decoded_err_to_contract_custom_error = err.as_decoded_error::<SomeCustomError>().unwrap();
    println!(
        "Decoded err to contract custom error: {:?}",
        decoded_err_to_contract_custom_error
    );

    println!("\n=== 新增 as_decoded_interface_error()方法");
    println!("如果不知道revert的是哪个自定义error的话可以使用as_decoded_interface_error()方法");
    println!("as_decoded_interface_error()方法返回一个enum，包含了所有的error");
    println!(
        "新增类型 ContractName::ErrorsErrors enum 包含了该Contract的所有error，\
        \n可以用与匹配as_decoded_interface_error()返回的类型"
    );
    // let decoded_err_to_interface_error = err.as_decoded_interface_error::<Errors::ErrorsErrors>().unwrap();
    let decoded_err_to_interface_error = err.as_decoded_interface_error().unwrap();
    match decoded_err_to_interface_error {
        ErrorsErrors::SomeCustomError(err) => {
            println!("Decoded as: {:#?}", err);
        }
        ErrorsErrors::AnotherError(err) => {
            println!("Decoded as: {:#?}", err);
        }
    }

    Ok(())
}
