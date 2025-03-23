// extern crate core;

// use core::slice::SlicePattern;
use alloy::consensus::TxEnvelope;
use alloy::eips::Encodable2718;
use alloy::network::{EthereumWallet, TransactionBuilder};
use alloy::primitives::private::serde::{Deserialize, Serialize};
use alloy::primitives::utils::{eip191_message, parse_units};
use alloy::primitives::{B256, TxHash, TxKind, U256, address, eip191_hash_message, keccak256};
use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use alloy::rpc::client::{RpcCall, RpcClient};
use alloy::rpc::types::mev::{BundleStats, EthBundleHash, EthSendBundle};
use alloy::signers::Signer;
use alloy::signers::local::{LocalSigner, PrivateKeySigner};
// use alloy::transports::http::Client;
// use alloy::transports::http::Http;
use alloy::transports::http::reqwest::header::HeaderValue;
use alloy::transports::http::{
    Http,
    HyperClient,
    HyperResponse,
    HyperResponseFut,
    hyper,
    // hyper::body::Body,
    hyper_util::{
        client::legacy::{Client, Error},
        rt::TokioExecutor,
    },
};
use hyper_tls::HttpsConnector;

// use alloy::rlp::Buf;
use alloy::{hex, sol};
use eyre::Result;
// use futures_util::FutureExt;
use futures_util::StreamExt;
use http_body_util::{BodyExt, Full};
use std::path::PathBuf;
use std::thread::sleep;
use tower::{Layer, Service};

// 使用abi, 合约地址, provider创建contract instance 和合约交互
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    OpenSpaceNFT,
    "src/abi/OpenspaceNFT.json"
}

// sol!(
//     #[allow(missing_docs)]
//     #[sol(rpc)]
//     OpenspaceNFT,
//     "src/abi/OpenspaceNFT.json"
// );

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 订阅监听交易
    // ws url
    let rpc_url = "wss://ethereum-sepolia-rpc.publicnode.com";
    let ws = WsConnect::new(rpc_url);
    let pubsub_provider = ProviderBuilder::new().on_ws(ws).await?;
    let full_pending_tx_subscription = pubsub_provider
        .subscribe_full_pending_transactions()
        .await?;

    let http_url = "https://ethereum-sepolia-rpc.publicnode.com".parse()?;
    let nft_owner_address = address!("0xD0148b6eB2471F86126Cfe4c4716ab71889131ff");
    // let not_owner_address = address!("0xc213d510fe60552A27F29842729bD28393CBFEe7");
    let not_owner_address = address!("0x5520B9B4e135F061C72CE780ED3fE215B04Dd407");
    let nft_contract_address = address!("0x24C263EB836bcACab2529Ec30a02262617737025");
    // let enable_presale_selector = "0xa8eac492";
    let enable_presale_selector_without_0x = "a8eac492";
    let enable_presale_selector_bytes = hex::decode(enable_presale_selector_without_0x)?;
    let (target_tx_hash_sender, mut rx) = tokio::sync::mpsc::channel::<TxHash>(1);

    tokio::spawn(async move {
        let mut tx_stream = full_pending_tx_subscription.into_stream();
        while let Some(tx) = tx_stream.next().await {
            // println!("Full pending transaction details: {:#?}", tx);
            // 过滤签名者是否为 NFT owner地址
            if !tx.inner.signer().eq(&nft_owner_address)
                && !tx.inner.signer().eq(&not_owner_address)
            {
                // 测试，先打印出来
                println!("skip, not NFT owner address: {:?}", tx.inner.signer());
                // 跳过不是 NFT owner地址的交易, 这里先不跳过
                continue;
            }
            println!("match owner address -> {:?}", tx.inner.signer());

            // if let  inner_tx = tx.inner.inner() {
            //     inner_tx.is_eip1559();
            //     println!("Full pending transaction details: {:#?}", tx);
            // }
            match tx.inner.inner() {
                TxEnvelope::Legacy(tx_legacy) => {
                    println!("\n=== legacy ===");
                    println!("full pending tx: {:?}", tx_legacy);
                    if let TxKind::Call(contract_address) = tx_legacy.tx().to {
                        println!("current contract address: {:#?}", contract_address);
                        // check if contract_address is NFT contract address
                        if !contract_address.eq(&nft_contract_address) {
                            println!("skip, not NFT contract address: {:?}", nft_contract_address);
                            // 跳过不是 NFT contract地址的交易
                            continue;
                        }

                        // check if function selector is enablePresale(), target selector: "0xa8eac492"
                        let data = &tx_legacy.tx().input;
                        if data.is_empty() {
                            println!(
                                "skip, empty input_data: {:#?}, is a transfer tx, skip",
                                data
                            );
                            // 跳过没有input data的交易, 转账交易
                            continue;
                        }
                        // 取前4个字节，使用[u8]表示，u8 是一个字节
                        let selector = &data[0..4];
                        let hexed_selector = hex::encode(selector);
                        println!(
                            "current selector: 0x{}, bytes selector: {:?}",
                            hexed_selector, selector
                        );
                        if selector != enable_presale_selector_bytes.as_slice() {
                            println!(
                                "skip, not enablePresale() selector: 0x{}, bytes selector: {:?}",
                                enable_presale_selector_without_0x,
                                enable_presale_selector_bytes.as_slice()
                            );
                            // 跳过不是 enablePresale() selector的交易
                            continue;
                        }
                        target_tx_hash_sender.send(*tx.inner.hash()).await.unwrap();
                        return;
                    }
                }
                TxEnvelope::Eip2930(_tx_2930) => {
                    println!("=== eip2930 ===");
                    continue;
                    // println!("full pending tx: {:#?}", _tx_2930);
                }
                TxEnvelope::Eip1559(tx_1559) => {
                    println!("\n=== eip1559 ===");
                    println!("full pending tx: {:?}", tx_1559);
                    if let TxKind::Call(contract_address) = tx_1559.tx().to {
                        println!("current contract address: {:#?}", contract_address);
                        // check if contract_address is NFT contract address
                        if !contract_address.eq(&nft_contract_address) {
                            println!(
                                "skip, not NFT contract address: {:#?}",
                                nft_contract_address
                            );
                            // 跳过不是 NFT contract地址的交易
                            continue;
                        }

                        // check if function selector is enablePresale(), target selector: "0xa8eac492"
                        let data = &tx_1559.tx().input;
                        if data.is_empty() {
                            println!(
                                "skip, empty input_data: {:#?}, is a transfer tx, skip",
                                data
                            );
                            // 跳过没有input data的交易, 转账交易
                            continue;
                        }
                        // 取前4个字节，使用[u8]表示，u8 是一个字节
                        let selector = &data[0..4];
                        let hexed_selector = hex::encode(selector);
                        println!(
                            "current selector: 0x{}, bytes selector: {:?}",
                            hexed_selector, selector
                        );
                        if selector != enable_presale_selector_bytes.as_slice() {
                            println!(
                                "skip, not enablePresale() selector: 0x{}, bytes selector: {:?}",
                                enable_presale_selector_without_0x,
                                enable_presale_selector_bytes.as_slice()
                            );
                            // 跳过不是 enablePresale() selector的交易
                            continue;
                        }

                        target_tx_hash_sender.send(*tx.inner.hash()).await.unwrap();
                        return;
                    }
                }
                TxEnvelope::Eip4844(_tx_4844) => {
                    println!("=== eip4844 ===");
                    continue;
                    // println!("full pending tx: {:#?}", _tx_4844);
                }
                TxEnvelope::Eip7702(_tx_7702) => {
                    println!("=== eip7702 ===");
                    continue;
                    // println!("full pending tx: {:#?}", _tx_7702);
                }
            }
        }
    });
    // handle.await?;

    // 2. 创建signer wallet
    // 读取 本地 keystore文件
    let keystore_file_path = PathBuf::from(std::env::var("KEYSTORE_PATH")?);
    // 读取password, 解锁keystore 创建signer
    let keystore_signer =
        LocalSigner::decrypt_keystore(keystore_file_path, std::env::var("KEYSTORE_PWD")?)?;
    let private_signer = PrivateKeySigner::from(keystore_signer.clone());
    // 创建wallet
    let wallet = EthereumWallet::from(keystore_signer.clone());

    let provider = ProviderBuilder::new()
        .wallet(wallet.clone())
        .on_http(http_url);

    // let flashbots_url = "https://relay-sepolia.flashbots.net".parse()?;
    // let flashbots_provider = ProviderBuilder::new()
    //     .wallet(wallet.clone())
    //     .on_http(flashbots_url);

    // Create a new Hyper client.
    // let hyper_client =
    //     Client::builder(TokioExecutor::new()).build_http::<Full<hyper::body::Bytes>>();

    // support tls
    let https = HttpsConnector::new();
    let hyper_client =
        Client::builder(TokioExecutor::new()).build::<_, Full<hyper::body::Bytes>>(https);

    // Use tower::ServiceBuilder to stack layers on top of the Hyper client.
    let service = tower::ServiceBuilder::new()
        .layer(FlashbotsSignatureLayer::new(private_signer))
        .service(hyper_client);

    // Instantiate the HyperClient with the stacked layers.
    let flashbots_http_url = "https://relay-sepolia.flashbots.net".parse()?;
    let layer_transport = HyperClient::<Full<hyper::body::Bytes>, _>::with_service(service);
    let http = Http::with_client(layer_transport, flashbots_http_url);

    // Create a new RPC client with the Hyper transport.
    let flashbots_rpc_client = RpcClient::new(http, true);
    let flashbots_provider = ProviderBuilder::new().on_client(flashbots_rpc_client);

    // 3. 构造自己的交易
    let nft_instance = OpenSpaceNFT::new(nft_contract_address, flashbots_provider.clone());
    // 设置抢购nft所需的eth
    let value = parse_units("0.01", "ether")?;
    let u256_amount = value.into();

    // let tx = nft_instance
    //     .presale(U256::from(10))
    //     .value(u256_amount)
    //     .into_transaction_request()
    //     .build_typed_tx()
    //     .unwrap();

    // 缺少 ["nonce", "gas_limit", "max_fee_per_gas", "max_priority_fee_per_gas"]
    // 调用into_transaction_request时只填充了to data value, 不会自动从provider查询剩下的字段
    let nonce = provider.get_transaction_count(not_owner_address).await?;

    let tx_envelope = nft_instance
        .presale(U256::from(10))
        .value(u256_amount)
        .nonce(nonce)
        .gas(3000000)
        .max_fee_per_gas(5_000_000_000u128)
        .max_priority_fee_per_gas(2_000_000_000u128)
        .into_transaction_request()
        .build(&wallet)
        .await?;
    let tx_encoded = tx_envelope.encoded_2718();
    // rlp编码, rlp编码之后才是十六进制字符传，以0x开头
    // flashbots文档示例 0x开头，应该是rlp编码
    // {
    //   "method": "eth_sendBundle",
    //   "params": [
    //     {
    //       "txs": ["0x123abc...", "0x456def..."],
    //     }
    //   ]
    // }
    let rlp_hex = hex::encode_prefixed(tx_encoded);

    // let tx_hash = provider.client().request("eth_sendRawTransaction", (rlp_hex,)).await?;

    // let signed_tx = wallet.sign_transaction(tx.clone().into()).await?;
    // let signed_tx = wallet.sign_transaction_from(wallet.default_signer_address(), tx.clone()).await?;
    // let raw_tx = hex::decode(signed_tx)?;

    // 接收匹配的tx hash
    let target_tx_hash = rx.recv().await.unwrap();
    // 获取目标交易的原始交易
    // eth_getRawTransactionByHash返回的是 rlp 编码过后的，即 0x......
    // 但是 alloy 把它解码成了Bytes,
    // 所以需要 hex::encode_prefixed()
    // TODO: 确认是否是这样
    let target_raw_tx = provider
        .get_raw_transaction_by_hash(target_tx_hash)
        .await?
        .unwrap();
    // let hexed_target_raw_tx = hex::encode_prefixed(target_raw_tx);
    let target_block_number = provider.get_block_number().await?;

    // 构造一个新的provider with X-Flashbots-Signature
    // Set the X-Flashbots-Signature header.
    // let mut headers = HeaderMap::new();

    // let header_value = format!("{}:{}}", wallet.default_signer_address(), );
    // headers.insert(
    //     "X-Flashbots-Signature",
    //     HeaderValue::from_static("deadbeef"),
    // );

    // Create the reqwest::Client with the AUTHORIZATION header.
    // let client_with_auth = Client::builder().build()?;

    // Create the HTTP transport.
    // let http = Http::with_client(client_with_auth, rpc_url);
    // let rpc_client = RpcClient::new(http, false);
    // let with_headers_flashbots_provider =
    //     ProviderBuilder::new();

    // 4. 构造 bundle
    let mut bundle = EthSendBundle::default();
    bundle.txs.push(target_raw_tx);
    // FIXME: 这里如何构造符合预期的数据，get_raw_transaction_by_hash获取的数据是刚好符合预期的
    // bundle.txs.push(tx_encoded.into());
    bundle.block_number = target_block_number;
    // 5. 发送 bundle
    // let request = flashbots_provider.client().make_request("eth_sendBundle", [bundle]);
    // let resp = flashbots_provider.client().request::<RpcCall<(EthSendBundle,), EthBundleHash>>("eth_sendBundle", (bundle,));
    let bundle_hash = flashbots_provider.send_bundle(bundle).await?;
    println!("Bundle Hash: {:?}", bundle_hash);

    let hexed_block_number = format!("0x{:x}", target_block_number);

    // 6. 查询 bundle 状态
    let param = GetBundleStatsParam {
        bundle_hash: bundle_hash.bundle_hash,
        block_number: hexed_block_number,
    };

    loop {
        // TODO: param是否会有底层没存释放不及时导致内存泄漏的风险？内存分配器（malloc），或者 jemalloc 带来的OOM风险?
        // 蚂蚁集团：如何在生产环境排查Rust内存占用过高问题
        // https://rustmagazine.github.io/rust_magazine_2021/chapter_5/rust-memory-troubleshootting.html
        let bundle_stats = flashbots_provider
            .get_bundle_stats_v2(param.clone())
            .await?;
        match bundle_stats {
            BundleStats::Unknown => {
                println!("Bundle status: Unknown");
            }
            BundleStats::Seen(stats) => {
                println!("Bundle status Seen: {:?}", stats);
            }
            BundleStats::Simulated(stats) => {
                println!("Bundle status Simulated: {:#?}", stats);
                return Ok(());
            }
        }

        sleep(std::time::Duration::from_secs(1));
    }
}

// 为 Provider 扩展 send_bundle 方法
trait FlashbotsProviderExt: Provider {
    fn send_bundle(
        &self,
        eth_send_bundle: EthSendBundle,
    ) -> RpcCall<(EthSendBundle,), EthBundleHash> {
        self.client()
            .request("eth_sendBundle", (eth_send_bundle,))
            .into()
    }
    //   fn get_raw_transaction_by_hash(&self, hash: TxHash) -> ProviderCall<(TxHash,), Option<Bytes>> {
    //         self.client().request("eth_getRawTransactionByHash", (hash,)).into()
    //     }

    fn get_bundle_stats_v2(
        &self,
        param: GetBundleStatsParam,
    ) -> RpcCall<(GetBundleStatsParam,), BundleStats> {
        self.client()
            .request("flashbots_getBundleStatsV2", (param,))
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetBundleStatsParam {
    bundle_hash: B256,
    // block_number: BlockNumber,
    block_number: String,
}

// 为所有 Provider 实现扩展 trait
impl<P: Provider> FlashbotsProviderExt for P {}

/// === modify layer
// 自定义 Layer, 用于添加flashbots header X-Flashbots-Signature
#[derive(Clone)]
struct FlashbotsSignatureLayer {
    // wallet: EthereumWallet,
    wallet: PrivateKeySigner,
}

impl FlashbotsSignatureLayer {
    fn new(wallet: PrivateKeySigner) -> Self {
        Self { wallet }
    }
}

impl<S> Layer<S> for FlashbotsSignatureLayer {
    type Service = FlashbotsSignatureService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        FlashbotsSignatureService {
            inner,
            wallet: self.wallet.clone(),
        }
    }
}

/// === 实现flashbots signature service 来添加header
// 自定义 Service
#[derive(Clone)]
struct FlashbotsSignatureService<S> {
    inner: S,
    // wallet: EthereumWallet,
    wallet: PrivateKeySigner,
}

impl<S, B> Service<hyper::Request<B>> for FlashbotsSignatureService<S>
where
    S: Service<hyper::Request<B>, Response = HyperResponse, Error = Error>
        + Clone
        + Send
        + Sync
        + 'static,
    S::Future: Send,
    S::Error: std::error::Error + Send + Sync + 'static,
    // B: From<Vec<u8>> + Send + 'static + Clone + Sync + std::fmt::Debug,
    B: hyper::body::Body<Data = hyper::body::Bytes> + Send + 'static + Clone + Sync,
    <B as hyper::body::Body>::Error: std::fmt::Debug,
{
    type Response = HyperResponse;
    type Error = Error;
    type Future = HyperResponseFut;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[allow(unused_mut)]
    fn call(&mut self, mut req: hyper::Request<B>) -> Self::Future {
        // async move {
        let mut cloned_req = req.clone();
        let wallet = self.wallet.clone();
        // 获取请求体
        // let mut req_body: Vec<u8> = req.into_body().clone().into();
        // let rt = tokio::runtime::Runtime::new().unwrap();

        // let whole_body_c = rt.block_on(req.collect()).unwrap();
        let whole_body_c = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(req.collect())
        })
        .unwrap();
        let whole_body = whole_body_c.to_bytes();
        println!("Body: {}", String::from_utf8_lossy(&whole_body));
        // let body_bytes = hyper::body::to_bytes(req.body_mut()).await?;
        // let body_bytes = whole_body.iter().collect::<Vec<u8>>();
        let hashed_body = keccak256(whole_body.as_ref());
        println!("Hashed Body: {:?}", hex::encode(hashed_body));
        // bug fix
        // 参考: https://github.com/onbjerg/ethers-flashbots/blob/64a0ac980702037dc7499ffdd46cb6bf406442f3/src/relay.rs#L84
        // 给body签名的步骤是:
        // 1. hash(body)
        // 2. hex hashed_body
        // 3. add 0x prefix to hexed_hashed_body
        // 4. sign_message(0xhexed_hashed_body) 到这里走eth eip191 sign_message的流程就可以了
        let hexed_body = hex::encode(hashed_body);
        let hexed_body_with_perfix = format!("0x{}", hexed_body);

        // 生成签名
        let signature = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(wallet.sign_hash(&eip191_hash_message(&hexed_body_with_perfix)))
            //sign_message  ==  self.sign_hash(&eip191_hash_message(message)).await
        })
        .unwrap();
        println!("EIP-191 Message: {:?}", eip191_message(hashed_body));
        println!(
            "hexed EIP-191 Message: {:?}",
            hex::encode(eip191_message(hashed_body))
        );

        println!("Signature: {:?}", hex::encode(signature.as_bytes()));
        // let signature = rt.block_on(wallet.sign_hash(&eip191_hash_message(hashed_body))).unwrap();
        // let signature = wallet.sign_hash(&eip191_hash_message(hashed_body)).await.unwrap();
        let header_value = format!(
            "{}:0x{}",
            wallet.address(),
            hex::encode(signature.as_bytes())
        );
        println!("Header: {}", header_value);

        // === 验证一下签名，看是否正常
        let recover_addr = signature
            .recover_address_from_msg(&hexed_body_with_perfix)
            .unwrap();
        println!("Recover address from msg: {:?}", recover_addr);
        let recover_addr_from_hash = signature
            .recover_address_from_prehash(&eip191_hash_message(&hexed_body_with_perfix))
            .unwrap();
        println!(
            "Recover address from final hash: {:?}",
            recover_addr_from_hash
        );

        // 修改请求头
        cloned_req.headers_mut().insert(
            "X-Flashbots-Signature",
            HeaderValue::from_str(&header_value).unwrap(),
        );

        // 重新设置 body
        // TODO:?
        // *req.body_mut() = Body::(body_bytes);

        // 调用底层服务
        // let response = self.inner.call(cloned_req).await.unwrap();
        // Ok(response)

        let fut = self.inner.call(cloned_req);

        Box::pin(fut)
        // }
        // .boxed()
    }
}
