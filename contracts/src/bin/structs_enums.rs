use alloy::{dyn_abi::SolType, primitives::U256, sol};
use eyre::Result;

// Generates Rust bindings for Solidity structs, enums and type aliases.
sol! {
    #[allow(missing_docs)]
    #[derive(Debug)]
    /// Foo
    struct Foo {
        uint256 a;
        uint64 b;
        Bar greater;
    }

    #[allow(missing_docs)]
    #[derive(Debug)]
    /// Bar
    enum Bar {
        A,
        B,
    }
}

fn main() -> Result<()> {
    // create an instance of the struct
    let foo = Foo {
        a: U256::from(1),
        b: 2_u64,
        greater: Bar::A,
    };
    println!("{foo:#?}");

    let encoded_bytes = Foo::abi_encode(&foo);
    println!("Rust Binding Type -> abi.encode bytes, {:?}", encoded_bytes);

    let decode_to_rust_binding_type = Foo::abi_decode(&encoded_bytes, true)?;
    println!(
        "abi.encode bytes -> Rust Binding Type, {:?}",
        decode_to_rust_binding_type
    );

    // 如果我通过get_storage_at只拿到后两个字段的数据 bytes 即 Vec<u8>
    // 需要手动填充完整的Foo所需数据再反序列化
    // TODO: 需要注意的是get_storage_at返回的数据一定是32字节的Vec<u8>
    // 存储布局和abi编码不对等
    let partial_data: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, // b = 2 (uint64)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, // greater = Bar::B (假设 Bar::B 编码为 1)
    ];

    let a_default = U256::from(0);
    let a_bytes = a_default.to_be_bytes_vec();

    // 将 b 编码为32字节
    let b = u64::from_be_bytes(partial_data[0..8].try_into()?);
    let mut b_bytes = vec![0u8; 24]; // 前24byte 补0
    b_bytes.extend_from_slice(&b.to_be_bytes()); // 追加后8字节数据

    // 将 greater编码为32字节
    // 前31字节补0
    let greater = partial_data[8];
    let mut greater_bytes = vec![0u8; 31];
    greater_bytes.push(greater);

    // 构造完整的ABI编码数据
    let mut full_foo_bytes = Vec::new();
    // 填充a
    full_foo_bytes.extend_from_slice(&a_bytes);
    // 填充获取的后两个字段的数据
    // 填充b
    full_foo_bytes.extend_from_slice(&b_bytes);
    // 填充 greater
    full_foo_bytes.extend_from_slice(&greater_bytes);
    // full_foo_bytes.extend_from_slice(&partial_data);
    println!("full bytes: {:?}", full_foo_bytes);

    let decoded_to_foo = Foo::abi_decode(&full_foo_bytes, true)?;

    println!("\nDecoded Foo: {:#?}", decoded_to_foo);

    Ok(())
}
