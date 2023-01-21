use std::{collections::HashMap, error::Error};

use futures::future::select_all;
use tokio::net::TcpStream;

use crate::connection::Connection;

#[async_trait::async_trait]
pub trait Readable<T> {
    async fn readable(&self) -> Result<T, Box<dyn Error>>;
}

#[async_trait::async_trait]
impl Readable<usize> for Vec<TcpStream> {
    async fn readable(&self) -> Result<usize, Box<dyn Error>> {
        if self.is_empty() {
            return Err("no waitings".into());
        }

        match select_all(self.iter().enumerate().map(|(index, stream)| {
            Box::pin(async move {
                stream.readable().await?;

                Ok::<usize, Box<dyn Error>>(index)
            })
        }))
        .await
        {
            (Ok(index), _, _) => Ok(index),
            (Err(e), _, _) => Err(e),
        }
    }
}

#[async_trait::async_trait]
impl Readable<String> for HashMap<String, Connection> {
    async fn readable(&self) -> Result<String, Box<dyn Error>> {
        if self.is_empty() {
            return Err("no waitings".into());
        }

        match select_all(self.iter().map(|(key, conn)| {
            Box::pin(async move {
                conn.tcp_stream.readable().await?;

                Ok::<&str, Box<dyn Error>>(key)
            })
        }))
        .await
        {
            (Ok(key), _, _) => Ok(key.to_owned()),
            (Err(e), _, _) => Err(e),
        }
    }
}
