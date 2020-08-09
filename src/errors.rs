use crate::bencode::Error as BencodeError;
use async_std::io::Error as IoError;
use async_std::sync::RecvError as AsyncRecvError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("io error {0}")]
    Io(#[from] IoError),
    #[error(transparent)]
    Bencode(#[from] BencodeError),
    #[error(transparent)]
    AsyncRecv(#[from] AsyncRecvError),
    #[error("invalid bencode packet")]
    InvalidPacket,
    #[error("invalid udp socket")]
    InvalidUdpSocket,
    #[error("invalid tcp socket")]
    InvalidTcpStream,
    #[error("bencode dict not found '{0}'")]
    DictNotFound(String),
    #[error("invlaid reply 'nodes' length={0}")]
    InvalidNodesLength(usize),
    #[error("invalid token")]
    InvalidToken,
    #[error("remote peer not supporting bittorrent protocol")]
    PeerNotSupportBittorrentProtocol,
    #[error("remote peer not supporting extension protocol")]
    PeerNotSupportExtensionProtocol,
    #[error("invalid bittorrent header response")]
    InvalidBittorrentHeaderResponse,
    #[error("metadata size too long")]
    MetadataSizeTooLong,
    #[error("negative metadata size")]
    NegativeMetadataSize,
    #[error("invalid piece")]
    InvalidPiece,
    #[error("index not found")]
    IndexNotFound,
    #[error("metadata checksum mismatch")]
    MetadataChecksum,
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
