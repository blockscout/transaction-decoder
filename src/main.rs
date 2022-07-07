use actix_web::{web, App, HttpServer, Result};
use serde::Deserialize;
extern crate serde_json;
use serde_json::Value;

//extern crate reqwest;
use reqwest;
// This struct represents state

#[derive(Deserialize)]
struct ContractCall {
    contract: String,
    txn: String,
}
/*
#[derive(Deserialize)]
struct TxnInfoLog {
    address: String,
    data: String,
    index: u32,
    topics: Vec<String>,
}

#[derive(Deserialize)]
struct TxnInfoResult {
    confirmations: String,
    from: String,
    gasLimit: u32,
    gasPrice: u32,
    gasUsed: u32,
    hash: String,
    input: String,
    timeStamp: u32,
    logs: Vec<TxnInfoLog>,
    next_page_params: String,
    revertReason: String,
    success: bool,
    to: String,
    value: u32,
}

#[derive(Deserialize)]
struct TxnInfoResponse {
    message: String,
    result: TxnInfoResult,
    status: i32,
}
*/

// curl -X POST --data '{"id":0,"jsonrpc":"2.0","method": "eth_blockNumber","params": []}'
/// extract `Contract` using serde

async fn get_txn_info(txn: &String) -> String {
    let res = reqwest::get(format!(
        "https://blockscout.com/eth/mainnet/api?module=transaction&action=gettxinfo&txhash={}",
        txn
    ))
    .await;

    let res = match res {
        Ok(r) => r,
        Err(err) => panic!("{}", err),
    };

    let res = res.text().await;

    let res = match res {
        Ok(r) => r,
        Err(err) => panic!("{}", err),
    };

    res
}

fn get_txn_input_bytes(txn: &String) -> String {
    let data: Value = serde_json::from_str(&txn).unwrap();
    let obj = data.as_object().unwrap();
    let s = String::from(format!(
        "{}",
        obj.get("result").unwrap().get("input").unwrap()
    ));
    let res = &s[1..11];
    res.to_string()
}

async fn get_txn_method(input: &String) -> String {
    //https://www.4byte.directory/signatures/?bytes4_signature=0xa7ca0ba3
    let res = reqwest::get(format!(
        "https://www.4byte.directory/api/v1/signatures/?hex_signature={}",
        input
    ))
    .await;

    let res = match res {
        Ok(r) => r,
        Err(err) => panic!("{}", err),
    };

    let res = res.text().await;

    let res = match res {
        Ok(r) => r,
        Err(err) => panic!("{}", err),
    };

    res
}

fn decode_methods(methods: &String) -> Vec<String> {
    let data: Value = serde_json::from_str(&methods).unwrap();
    let obj = data.as_object().unwrap();
    let cnt = obj.get("count").unwrap();
    println!("{}", cnt);
    let cnt = match cnt {
        Value::Number(n) => n,
        other => panic!("other"),
    };

    let cnt = match cnt.as_u64() {
        Some(x) => x,
        None => panic!("None"),
    };
    if cnt == 0 {
        return Vec::new();
    }

    let variants = obj.get("results").unwrap();

    let variants = match variants.as_array() {
        Some(v) => v,
        None => panic!("None"),
    };

    let mut results: Vec<String> = Vec::new();
    for variant in variants.iter() {
        let s = variant.get("text_signature").unwrap().to_string();
        println!("s.len: {}", s.len());
        results.push(s[1..(s.len() - 1)].to_string());
    }

    results
}

struct Method {
    full_name: String,
    name: String,
    args: Vec<String>,
}

fn process_method(method: String) -> Method {
    let first_b = match method.find("(") {
        Some(i) => i,
        None => panic!("Invalid method"),
    };
    let last_b = match method.find(")") {
        Some(i) => i,
        None => panic!("Invalid method"),
    };
    let name = method[..first_b].to_string();
    let args = method[first_b..last_b].split(',');
    let args: Vec<&str> = args.collect();
    let args: Vec<String> = args.iter().map(|&x| x.into()).collect();

    Method {
        name,
        args,
        full_name: method,
    }
}

fn match_method_in_contract(code: &String, methods: Vec<String>) -> String {
    let methods: Vec<Method> = methods
        .iter()
        .map(|m| process_method(m.to_string()))
        .collect();
    for method in methods {
        let name_match = code.find(&method.name);
        let name_match = match name_match {
            Some(name) => name,
            None => continue,
        };
        let first_b = name_match + method.name.len();
        let last_b = match code[first_b..].find(")") {
            Some(ind) => ind + first_b,
            None => continue,
        };
        for arg in method.args {
            match code[first_b..last_b].find(&arg) {
                Some(_) => continue,
                None => break,
            }
        }
        return method.full_name;
    }
    "".to_string()
}

async fn index(contract_call: web::Json<ContractCall>) -> Result<String> {
    let data = get_txn_info(&contract_call.txn).await;
    let input_bytes = get_txn_input_bytes(&data);
    let methods = get_txn_method(&input_bytes).await;
    let methods = decode_methods(&methods);
    let result = match_method_in_contract(&contract_call.contract, methods);
    if result.len() == 0 {
        return Ok("Method not found".to_string());
    }

    Ok(result)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::post().to(index)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
