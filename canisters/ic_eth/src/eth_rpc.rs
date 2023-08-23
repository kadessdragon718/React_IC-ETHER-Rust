use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

use crate::util::to_hex;

const HTTP_CYCLES: u128 = 100_000_000;
const MAX_RESPONSE_BYTES: u64 = 2048;

thread_local! {
    static NEXT_ID: RefCell<u64> = RefCell::default();
}

fn next_id() -> u64 {
    NEXT_ID.with(|next_id| {
        let mut next_id = next_id.borrow_mut();
        let id = *next_id;
        *next_id = next_id.wrapping_add(1);
        id
    })
}

fn get_rpc_endpoint(network: &str) -> &'static str {
    match network {
        "mainnet" => "https://cloudflare-eth.com/v1/mainnet",
        "goerli" => "https://ethereum-goerli.publicnode.com",
        "sepolia" => "https://rpc.sepolia.org",
        _ => panic!("Unsupported network: {}", network),
    }
}

/// Call an Ethereum smart contract.
pub async fn call_eth(network: &str, contract_address: String, data: Vec<u8>) -> String {
    let service_url = get_rpc_endpoint(network).to_string();
    let json_rpc_payload = serde_json::to_string(&JsonRpcRequest {
        id: next_id(),
        jsonrpc: "2.0".to_string(),
        method: "eth_call".to_string(),
        params: (
            EthCallParams {
                to: contract_address,
                data: to_hex(&data),
            },
            "latest".to_string(),
        ),
    })
    .expect("Error while encoding JSON-RPC request");

    let parsed_url = url::Url::parse(&service_url).expect("Service URL parse error");
    let host = parsed_url
        .host_str()
        .expect("Invalid service URL host")
        .to_string();

    let request_headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Host".to_string(),
            value: host.to_string(),
        },
    ];
    let request = CanisterHttpRequestArgument {
        url: service_url,
        max_response_bytes: Some(MAX_RESPONSE_BYTES),
        method: HttpMethod::POST,
        headers: request_headers,
        body: Some(json_rpc_payload.as_bytes().to_vec()),
        transform: Some(TransformContext::from_name("transform".to_string(), vec![])),
    };
    let result = match http_request(request, HTTP_CYCLES).await {
        Ok((r,)) => r,
        Err((r, m)) => panic!("{:?} {:?}", r, m),
    };

    let json: JsonRpcResult =
        serde_json::from_str(std::str::from_utf8(&result.body).expect("utf8"))
            .expect("JSON was not well-formatted");
    if let Some(err) = json.error {
        panic!("JSON-RPC error code {}: {}", err.code, err.message);
    }
    json.result.expect("Unexpected JSON response")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    id: u64,
    jsonrpc: String,
    method: String,
    params: (EthCallParams, String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct EthCallParams {
    to: String,
    data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JsonRpcResult {
    result: Option<String>,
    error: Option<JsonRpcError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: isize,
    message: String,
}
