use alloy::{
    hex,
    primitives::{I256, U256, Uint},
    sol,
    sol_types::SolCall,
};
use eyre::Result;

// Codegen from excerpt of Chainlink Aggregator interface.
// See: https://etherscan.io/address/0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419#code
sol!(
    #[allow(missing_docs)]
    #[derive(Debug, PartialEq, Eq)]
    function getRoundData(uint80 _roundId) external view returns (uint80 roundId, int256 answer, uint256 startedAt, uint256 updatedAt, uint80 answeredInRound);
);

fn main() -> Result<()> {
    // 解码到指定函数返回值类型
    let decoded_result = getRoundDataCall::abi_decode_returns(
        &hex!(
            "0000000000000000000000000000000000000000000000060000000000004716
             00000000000000000000000000000000000000000000000000000051faad1c80
             000000000000000000000000000000000000000000000000000000006669627b
             000000000000000000000000000000000000000000000000000000006669627b
             0000000000000000000000000000000000000000000000060000000000004716"
        ),
        true,
    );

    assert_eq!(
        decoded_result,
        Ok(getRoundDataReturn {
            roundId: Uint::<80, 2>::from(110680464442257327894_u128),
            answer: I256::from_dec_str("352098000000")?,
            startedAt: U256::from(1718182523),
            updatedAt: U256::from(1718182523),
            answeredInRound: Uint::<80, 2>::from(110680464442257327894_u128),
        })
    );

    println!("Decode Result:\n{:#?}", decoded_result?);

    Ok(())
}
