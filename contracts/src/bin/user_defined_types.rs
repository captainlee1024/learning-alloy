use alloy::{
    dyn_abi::SolType,
    primitives::{Address, U256},
    sol,
};
use eyre::Result;

sol! {
    type CustomType is uint256;
}

type Bytes32 = sol! { bytes32 };

// This is equivalent to the following:
// type B32 = alloy_sol_types::sol_data::FixedBytes<32>;

// User defined types
type CustomArrayOf<T> = sol! { T[] };
type CustomTuple = sol! { tuple(address, bytes, string) };
/// 1. 调用合约时
/// 链下（Rust 或其他客户端）：
/// 你需要将调用合约的参数 手动编码 为 ABI 格式（Vec<u8>）。
///
/// 使用 abi_encode 方法将 Rust 类型编码为 Solidity 的 ABI 格式。
///
/// 链上（Solidity 合约）：
/// Solidity 会自动将传入的 ABI 编码数据 解码 为合约函数的参数类型。
///
/// 你不需要在 Solidity 中手动调用 abi.decode。
///
/// 2. 合约返回结果时
/// 链上（Solidity 合约）：
/// Solidity 会自动将返回值 编码 为 ABI 格式（bytes）。
///
/// 你不需要在 Solidity 中手动调用 abi.encode。
///
/// 链下（Rust 或其他客户端）：
/// 合约返回的结果是 ABI 编码后的数据（Vec<u8>）。
///
/// 你需要 手动解码 这些数据，将其转换为 Rust 类型。
///
/// 使用 abi_decode 方法将 ABI 编码数据解码为 Rust 类型。
///
///
/// 在链下使用alloy调用链上合约时
/// Rust Binding Type Data -> abi.encode -> 链上   合约接收到之后-> abi.decode -> 参数列表对应的类型
///
/// 但是链上返回时
/// 合约返回列表 -> abi.encode -> 链下 alloy接收到之后需要我们手动解码到返回列表的类型上
///
/// sol! 宏可以生成Rust -> Solidity 的各种类型绑定，拿到链上返回的abi.encode数据之后(即Vec<u8>)
/// 可以使用对应的类型直接反序列化，省去手动解析过程
fn main() -> Result<()> {
    let b32_type = Bytes32::abi_encode(&[1; 32]);
    println!("Bytes32");
    println!("Rust -> Solidity, Encoded Bytes32: {:?}", b32_type);
    println!(
        "Solidity -> Rust, Decoded Bytes32: {:?}",
        Bytes32::abi_decode(&b32_type, true)?
    );

    let custom_type = CustomType(U256::from(1));
    let encoded_custom_bytes = CustomType::abi_encode(&custom_type);
    println!("\nCustom type");
    println!(
        "Rust -> Solidity, Encoded CustomType: {:?}",
        encoded_custom_bytes
    );

    let _decoded = CustomType::abi_decode(&encoded_custom_bytes, true)?;

    println!(
        "Solidity -> Rust, Decoded CustomType: {:?}",
        CustomType::abi_decode(&encoded_custom_bytes, true)?
    );

    let custom_array_of_type = CustomArrayOf::<sol!(bool)>::abi_encode(&vec![true, false]);
    println!("\nCustom Array type");
    println!(
        "Rust -> Solidity, Encoded Custom Array type: {:?}",
        custom_array_of_type
    );

    // 通过sol! 生成的Binding类型可以直接decode 合约中拿到的bytes数据到对应的类型，省略掉手动解析过程
    let _decoded = CustomArrayOf::<sol!(bool)>::abi_decode(&custom_array_of_type, true)?;
    println!(
        "Solidity -> Rust, Decoded Custom Array type: {:?}",
        CustomArrayOf::<sol!(bool)>::abi_decode(&custom_array_of_type, true)?
    );

    // 自定义类型的ABI编码
    let custom_tuple_type =
        CustomTuple::abi_encode(&(Address::ZERO, vec![0; 32], "hello".to_string()));
    println!("\ncustom tuple");
    println!("Rust -> Solidity, Encoded Typle: {:?}", custom_tuple_type);

    let _decoded = CustomTuple::abi_decode(&custom_tuple_type, true)?;
    // 自定义类型的ABI解码
    println!(
        "Solidity -> Rust, Decoded Tuple: {:?}",
        CustomTuple::abi_decode(&custom_tuple_type, true)?
    );

    Ok(())
}
