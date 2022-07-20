use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AbiMethod {
    pub inputs: Vec<AbiArg>,
    #[serde(rename = "type")]
    pub method_type: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AbiArg {
    #[serde(default)]
    pub indexed: bool,
    pub name: String,
    #[serde(rename = "type")]
    pub arg_type: String,
    #[serde(rename = "internalType")]
    pub internal_type: String,
}

#[derive(Deserialize, Debug)]
pub struct Request {
    pub txn: String,
    pub abi: Vec<AbiMethod>,
    pub contract: String,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub result: TransactionInput,
}

#[derive(Deserialize, Debug)]
pub struct TransactionInput {
    pub input: String,
}

#[derive(Serialize)]
pub struct Response {
    pub method: AbiMethod,
    pub line_number: u32,
}
