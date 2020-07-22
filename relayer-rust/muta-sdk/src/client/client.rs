use std::convert::TryInto;

use muta_protocol::{traits::ServiceResponse, types as muta_types};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

use super::request::GET_BLOCK_QUERY;
use super::rpc_types::{Block, RpcError};
use crate::util::u64_to_hex;

pub struct Config {
    pub url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8000/graphql".to_owned(),
        }
    }
}

pub struct HttpRpcClient {
    config: Config,
    client: reqwest::Client,
}

impl Default for HttpRpcClient {
    fn default() -> Self {
        let config = Config::default();
        Self::new(config)
    }
}

impl HttpRpcClient {
    pub fn new(config: Config) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    pub async fn raw<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        q: &T,
    ) -> Result<U, RpcError> {
        let mut resp: Value = self
            .client
            .post(&self.config.url)
            .json(&q)
            .send()
            .await?
            .json()
            .await?;
        if let Some(errs) = resp.get("errors") {
            return Err(RpcError::GraphQLError(errs.to_string()));
        }
        // dbg!(resp.to_string());
        Ok(serde_json::from_value(
            resp.get_mut("data")
                .ok_or(RpcError::DataIsNone)?
                .get_mut("getBlock")
                .ok_or(RpcError::DataIsNone)?
                .take(),
        )?)
    }

    pub async fn get_block(&self, height: Option<u64>) -> Result<muta_types::Block, RpcError> {
        let q = json!({
            "query": GET_BLOCK_QUERY,
            "variables": {
                "h": height.map(u64_to_hex),
            },
        });
        let rpc_block: Block = self.raw(&q).await?;
        Ok(rpc_block.try_into()?)
    }

    pub async fn get_transaction(
        &self,
        _tx_hash: muta_types::Hash,
    ) -> Result<muta_types::SignedTransaction, RpcError> {
        todo!()
    }

    pub async fn get_receipt(
        &self,
        _tx_hash: muta_types::Hash,
    ) -> Result<muta_types::Receipt, RpcError> {
        todo!()
    }

    pub async fn get_block_hook_receipt(
        &self,
        _height: u64,
    ) -> Result<muta_types::BlockHookReceipt, RpcError> {
        todo!()
    }

    pub async fn query_service(
        &self,
        _height: Option<u64>,
        _cycles_limit: Option<u64>,
        _cycles_price: Option<u64>,
        _caller: muta_types::Address,
        _service_name: String,
        _method: String,
        _payload: String,
    ) -> Result<ServiceResponse<String>, RpcError> {
        todo!()
    }

    pub async fn send_transaction(
        &self,
        _tx: muta_types::SignedTransaction,
    ) -> Result<muta_types::Hash, RpcError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn client_get_block_works() {
        let client = HttpRpcClient::default();
        let res = client.get_block(None).await.unwrap();
        println!("{:?}", res);
    }
}
