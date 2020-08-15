use crate::bencode::Error as BencodeError;
use async_std::io::Error as AsyncIoError;
use async_std::sync::RecvError as AsyncRecvError;
use serde_json::error::Error as SerdeJsonError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    AsyncIo(#[from] AsyncIoError),
    #[error(transparent)]
    Bencode(#[from] BencodeError),
    #[error(transparent)]
    AsyncRecv(#[from] AsyncRecvError),
    #[error(transparent)]
    SerdeJson(#[from] SerdeJsonError),
    #[error("bencode dict not found '{0}'")]
    DictNotFound(String),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
