use crate::Bytes;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

use ethabi::Contract;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AbiMethod {
    pub inputs: Vec<AbiArg>,
    #[serde(rename = "type")]
    pub method_type: String,
    #[serde(default)]
    pub name: String,
}

impl AbiMethod {
    pub fn selector(&self) -> [u8; 4] {
        let mut hasher = Keccak256::new();
        hasher.update(self.signature().as_bytes());
        let res = hasher.finalize();

        let mut ans = [0u8; 4];
        ans.copy_from_slice(&res[0..4]);
        ans
    }

    fn signature(&self) -> String {
        let args = self
            .inputs
            .iter()
            .map(|x| x.arg_type.clone())
            .collect::<Vec<_>>()
            .join(",");
        format!("{}({})", self.name, args)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbiArg {
    #[serde(default)]
    pub indexed: bool,
    pub name: String,
    #[serde(rename = "type")]
    pub arg_type: String,
    pub internal_type: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Request {
    pub tx_hash: Bytes,
    pub abi: Vec<AbiMethod>,
    pub network: String,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    // pub result: TransactionInput,
    pub message: String,
    pub result: Option<TransactionInput>,
    pub status: String,
}

#[derive(Deserialize, Debug)]
pub struct TransactionInput {
    pub input: Bytes,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub method: AbiMethod,
}
