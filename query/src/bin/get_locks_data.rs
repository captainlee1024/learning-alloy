use alloy::{
    network::EthereumWallet,
    node_bindings::Anvil,
    primitives::{Address, U64, U256, keccak256},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    sol,
};
use eyre::Result;

sol! {
    #[allow(missing_docs)]
    // solc v0.8.26; solc Counter.sol --viz-ir --optimize --bin
    #[sol(rpc, bytecode="608060405234801561000f575f80fd5b505f5b600b8110156100e6575f60405180606001604052808360016100349190610100565b6001600160a01b031681526020018361004e426002610119565b6100589190610130565b6001600160401b03168152602001610071846001610100565b61008390670de0b6b3a7640000610119565b90528154600181810184555f93845260209384902083516002909302018054948401516001600160401b0316600160a01b026001600160e01b03199095166001600160a01b03909316929092179390931781556040909101519082015501610012565b50610143565b634e487b7160e01b5f52601160045260245ffd5b80820180821115610113576101136100ec565b92915050565b8082028115828204841417610113576101136100ec565b81810381811115610113576101136100ec565b603e8061014f5f395ff3fe60806040525f80fdfea264697066735822122061c6be102b852df2191c9d69e810dfdc06e005d40a4ee6dd27462a263b7c273b64736f6c634300081a0033")]
    contract esRNT {
        struct LockInfo {
            address user;
            uint64 startTime;
            uint256 amount;
        }

        LockInfo[] private _locks;

        constructor() {
            for (uint256 i = 0; i < 11; i++) {
                _locks.push(LockInfo(address(uint160(i + 1)), uint64(block.timestamp * 2 - i), 1e18 * (i + 1)));
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Spin up a local Anvil node.
    // Ensure `anvil` is available in $PATH.
    let anvil = Anvil::new().block_time(1).try_spawn()?;
    let pk: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::new(pk);

    // Create a WebSocket provider.
    // let ws = WsConnect::new(anvil.ws_endpoint());
    // let provider = ProviderBuilder::new().wallet(wallet).on_ws(ws).await?;
    // use http client
    let http_provider = ProviderBuilder::new()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    let contract = esRNT::deploy(http_provider.clone()).await?;

    println!("Deployed esRNT contract at: {}", contract.address());

    let es_rnt_address = contract.address();
    let lock_info_slot = U256::from(0);

    // 默认在latest block上调用, .block() 可以设置一个区块
    let length = http_provider
        .get_storage_at(*es_rnt_address, lock_info_slot)
        .await?;
    println!(
        "get the lock info length: {:?} from the lockInfo slot 0",
        length
    );

    let lock_info_slot_data =
        U256::from_be_bytes(keccak256(lock_info_slot.to_be_bytes_vec()).into());

    for i in 0..length.to::<u64>() {
        // 元素下标
        let index = U256::from(i);

        // 计算第i个元素第一个slot的位置
        let slot_i_1 = lock_info_slot_data + (index * U256::from(2));
        // 第i个元素第二个slot的位置
        let slot_i_2 = slot_i_1 + U256::from(1);

        // let slot_i_1_data = http_provider
        //     .get_storage_at(*es_rnt_address, slot_i_1)
        //     .await?;
        // let slot_i_1_bytes: [u8; 32] = slot_i_1_data
        //     .to_be_bytes_vec()
        //     .try_into()
        //     .expect("failed to convert index i, slot 1 to [u8; 32]");
        // // let slot_i_1_bytes: [u8; 32] = (*slot_i_1_data).into();
        // // 拿到数据拆分出 address user, uint64 startTime
        // let user_addr = Address::from_slice(&slot_i_1_bytes[12..32]);
        // // let start_time_bytes: [u8; 8] = slot_i_1_bytes[4..12].try_into()?;
        // // let start_time = u64::from_be_bytes(start_time_bytes);
        // let start_time = U64::from_be_slice(&slot_i_1_bytes[4..12]);
        //
        // let slot_i_2_data = http_provider
        //     .get_storage_at(*es_rnt_address, slot_i_2)
        //     .await?;
        //
        // let slot_i_2_bytes: [u8; 32] = slot_i_2_data
        //     .to_be_bytes_vec()
        //     .try_into()
        //     .expect("failed to convert index i, slot 2 to [u8; 32]");
        // let amount = U256::from_be_bytes(slot_i_2_bytes);

        // 简化一些步骤
        // 获取改元素第一个slot的数据
        let slot_i_1_data = http_provider
            .get_storage_at(*es_rnt_address, slot_i_1)
            .await?;
        // 转换为bytes 即 Vec<u8> 无需进一步转换为bytes32 即 [u8; 32]
        let slot_i_1_bytes = slot_i_1_data.to_be_bytes_vec();
        // 解析地址 20字节
        let user_addr = Address::from_slice(&slot_i_1_bytes[12..32]);
        // 解析startTime 8字节
        let start_time = U64::from_be_slice(&slot_i_1_bytes[4..12]);
        // 获取该元素第二个slot的数据
        let slot_i_2_data = http_provider
            .get_storage_at(*es_rnt_address, slot_i_2)
            .await?;
        // 转换为bytes
        let slot_i_2_bytes = slot_i_2_data.to_be_bytes_vec();
        // 解析出 amount uint256, 32个字节，这里get_storage_at返回的数据刚好为
        // uint256 类型大小，而且alloy返回的数据刚好使用U256表示
        // 所以巧合导致slot_i_data 刚好等价于 amount，包括数据类型
        let amount = U256::from_be_slice(&slot_i_2_bytes);

        println!(
            "locks[{}]: user: {:?}, startTime: {:?}, amount: {:?}",
            i, user_addr, start_time, amount
        );
    }

    Ok(())
}
