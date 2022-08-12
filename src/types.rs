use crate::DisplayBytes;
use serde::{Deserialize, Serialize};

use ethabi::{Contract, Error, Function, Param, ParamType, Token};

type Result<T> = std::result::Result<T, Error>;

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
    pub method: Option<ResponseMethod>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseMethod {
    name: String,
    inputs: Vec<ResponseParam>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseParam {
    name: String,
    #[serde(rename(serialize = "internalType"))]
    internal_type: Option<String>,
    #[serde(rename(serialize = "type"))]
    kind: String,
    value: String,
}

trait ResponseDisplay {
    fn display(&self) -> String;
}

impl ResponseDisplay for ParamType {
    fn display(&self) -> String {
        match self {
            ParamType::String => "string".to_string(),
            ParamType::Address => "address".to_string(),
            ParamType::Bytes => "bytes".to_string(),
            ParamType::Int(n) => format!("int{}", n),
            ParamType::Uint(n) => format!("uint{}", n),
            ParamType::Bool => "bool".to_string(),
            ParamType::Array(t) => format!("{}[]", t.display()),
            ParamType::FixedBytes(n) => format!("bytes{}", n),
            ParamType::FixedArray(t, n) => format!("{}[{}]", t.display(), n),
            ParamType::Tuple(_) => "tuple".to_string(),
        }
    }
}

fn format_bytes(bytes: &[u8]) -> String {
    format!(
        "0x{}",
        bytes
            .iter()
            .map(|x| format!("{:x?}", x))
            .collect::<Vec<String>>()
            .join("")
    )
}

fn format_tokens(tokens: &[Token], sep: &str) -> String {
    tokens
        .iter()
        .map(|x| x.display())
        .collect::<Vec<String>>()
        .join(sep)
}

impl ResponseDisplay for Token {
    fn display(&self) -> String {
        match self {
            Token::String(s) => s.clone(),
            Token::Address(ad) => format!("{:?}", ad),
            Token::Bytes(b) | Token::FixedBytes(b) => format_bytes(b),
            Token::Int(n) | Token::Uint(n) => format!("{:?}", n),
            Token::Bool(b) => format!("{:?}", b),
            Token::FixedArray(tokens) | Token::Array(tokens) => {
                format!("[{}]", format_tokens(tokens, ""))
            }
            Token::Tuple(tokens) => format!("({})", format_tokens(tokens, ",")),
        }
    }
}

impl ResponseMethod {
    pub fn new(function: Option<Function>, data: &[u8]) -> Result<Option<ResponseMethod>> {
        match function {
            Some(f) => Ok(Some(ResponseMethod {
                name: f.name.clone(),
                inputs: f
                    .inputs
                    .iter()
                    .zip(f.decode_input(data)?.iter())
                    .map(|(input, token)| ResponseParam::new(input, token))
                    .collect(),
            })),
            None => Ok(None),
        }
    }
}

impl ResponseParam {
    fn new(param: &Param, token: &Token) -> ResponseParam {
        ResponseParam {
            name: param.name.clone(),
            kind: param.kind.display(),
            internal_type: param.internal_type.clone(),
            value: token.display(),
        }
    }
}
