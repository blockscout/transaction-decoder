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
#[serde(rename_all = "camelCase")]
pub struct ResponseParam {
    name: String,
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

impl ResponseDisplay for &Vec<u8> {
    fn display(&self) -> String {
        "0x".to_string() + &hex::encode(self)
    }
}

impl ResponseDisplay for &Vec<Token> {
    fn display(&self) -> String {
        self.iter()
            .map(|x| x.display())
            .collect::<Vec<String>>()
            .join(",")
    }
}

impl ResponseDisplay for Token {
    fn display(&self) -> String {
        match self {
            Token::String(s) => s.clone(),
            Token::Address(ad) => format!("{:?}", ad),
            Token::Bytes(b) | Token::FixedBytes(b) => b.display(),
            Token::Int(n) | Token::Uint(n) => format!("{:?}", n),
            Token::Bool(b) => format!("{:?}", b),
            Token::FixedArray(tokens) | Token::Array(tokens) => {
                format!("[{}]", tokens.display())
            }
            Token::Tuple(tokens) => format!("({})", tokens.display()),
        }
    }
}

impl ResponseMethod {
    pub fn new(function: Function, data: &[u8]) -> Result<ResponseMethod> {
        Ok(ResponseMethod {
            name: function.name.clone(),
            inputs: function
                .inputs
                .iter()
                .zip(function.decode_input(data)?.iter())
                .map(|(input, token)| ResponseParam::new(input.clone(), token.clone()))
                .collect(),
        })
    }
}

impl ResponseParam {
    fn new(param: Param, token: Token) -> ResponseParam {
        ResponseParam {
            name: param.name,
            kind: param.kind.display(),
            internal_type: param.internal_type,
            value: token.display(),
        }
    }
}

#[cfg(test)]
mod test {

    use ::ethabi::Token;

    use crate::types::ResponseDisplay;

    #[test]
    fn test_response_display_for_token() {
        let v: Vec<u8> = vec![1, 2, 3, 4];
        assert_eq!(Token::Bytes(v.clone()).display(), "0x01020304".to_string());
        assert_eq!(
            Token::FixedBytes(v.clone()).display(),
            "0x01020304".to_string()
        );

        let mut token_vec: Vec<Token> = vec![
            Token::Bytes(v.clone()),
            Token::Bytes(v.clone()),
            Token::Bytes(v.clone()),
        ];
        assert_eq!(
            Token::Array(token_vec.clone()).display(),
            "[0x01020304,0x01020304,0x01020304]".to_string()
        );

        token_vec = vec![Token::Array(token_vec.clone()), Token::Bytes(v.clone())];

        assert_eq!(
            Token::Tuple(token_vec.clone()).display(),
            "([0x01020304,0x01020304,0x01020304],0x01020304)".to_string()
        );
    }
}
