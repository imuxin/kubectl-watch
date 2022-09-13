use anyhow::Result;
use kube::{Client, Config, Error};
use std::convert::TryFrom;

pub async fn client(use_tls: bool) -> Result<Client, Error> {
    // init kube client
    let mut config = Config::infer().await.map_err(Error::InferConfig)?;
    if !use_tls {
        config.accept_invalid_certs = true;
    }

    Client::try_from(config)
}
