use alloy::{primitives::U256, rpc::json_rpc::ErrorPayload, sol};
use eyre::Result;

sol! {
    #[allow(missing_docs)]
    library Errors {
        error SomeCustomError(uint256 a);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Sample JSON error payload from an Ethereum JSON RPC response.
    let json = r#"{"code":3,"message":"execution reverted: ","data":"0x810f00230000000000000000000000000000000000000000000000000000000000000001"}"#;

    let payload: ErrorPayload = serde_json::from_str(json)?;

    let Errors::ErrorsErrors::SomeCustomError(value) = payload
        .as_decoded_error::<Errors::ErrorsErrors>(false)
        .unwrap();
    assert_eq!(value.a, U256::from(1));

    Ok(())
}
