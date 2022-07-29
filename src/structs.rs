use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AbiMethod {
    pub inputs: Vec<AbiArg>,
    #[serde(rename = "type")]
    pub method_type: String,
    #[serde(default)]
    pub name: String,
}

impl AbiMethod {
    pub fn function(&self) -> String {
        let v: Vec<String> = self
            .inputs
            .iter()
            .map(|x| format!("{}{}", x.arg_type, x.name))
            .collect();
        format!("{}({})", self.name, v.join(","))
    }

    pub fn function_signature(&self) -> String {
        let args = self
            .inputs
            .clone()
            .into_iter()
            .map(|x| x.arg_type)
            .collect::<Vec<_>>()
            .join(",");
        format!("{}({})", self.name, args)
    }
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

#[derive(Deserialize, Debug, Serialize)]
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
    //#[serde(with = "hex::serde")]
    pub input: String,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub method: AbiMethod,
    pub line_number: u32,
}
