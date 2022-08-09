use crate::DisplayBytes;
use serde::{Deserialize, Serialize};

use ethabi::{Contract, Function, Token};

#[derive(Deserialize, Debug, Serialize)]
pub struct Request {
    pub tx_hash: DisplayBytes,
    pub abi: Contract,
    pub network: String,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub message: String,
    pub result: Option<TransactionInput>,
    pub status: String,
}

#[derive(Deserialize, Debug)]
pub struct TransactionInput {
    pub input: DisplayBytes,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub method: Option<Function>,
    pub decoded_inputs: Option<Vec<Token>>,
}
