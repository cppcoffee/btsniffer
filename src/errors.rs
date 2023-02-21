use crate::Message;
use async_std::io::Error as AsyncIoError;
use bencode::Error as BencodeError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    AsyncIo(#[from] AsyncIoError),

    #[error(transparent)]
    Bencode(#[from] BencodeError),

    #[error("send fail, message: {0:?}")]
    Send(Message),

    #[error("bencode dict not found '{0}'")]
    DictNotFound(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
