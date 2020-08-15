use crate::{bencode, Error, Result};
use std::path::PathBuf;

#[derive(Debug)]
pub struct TorFile {
    pub name: String,
    pub length: i64,
}

#[derive(Debug)]
pub struct Torrent {
    pub name: String,
    pub length: i64,
    pub files: Vec<TorFile>,
}

// extract torrent inter path.
fn extract_path(value: &bencode::Value) -> Result<String> {
    let array = value.list()?;

    let mut paths = Vec::new();
    for s in array {
        paths.push(s.string()?);
    }

    paths
        .iter()
        .fold(PathBuf::new(), |pb, x| pb.join(x))
        .into_os_string()
        .into_string()
        .map_err(|e| Error::Other(format!("extract path fail, {:?}", e)))
}

// parse torrent included files.
fn extract_files(value: &bencode::Value) -> Result<TorFile> {
    let name: String;
    let mut length = 0_i64;

    let dict = value.dict()?;
    if let Some(x) = dict.get(b"path.utf-8".as_ref()) {
        name = extract_path(x)?;
    } else if let Some(x) = dict.get(b"path".as_ref()) {
        name = extract_path(x)?;
    } else {
        name = "".to_string();
    }

    if let Some(x) = dict.get(b"length".as_ref()) {
        length = x.integer()?;
    }

    Ok(TorFile { name, length })
}

// parse meta into Torrent instance.
pub fn from_bytes(meta: &[u8]) -> Result<Torrent> {
    let name: String;
    let mut length = 0_i64;

    let m = bencode::from_bytes(meta)?;
    let dict = m.dict()?;

    if let Some(s) = dict.get(b"name.utf-8".as_ref()) {
        name = s.string()?.to_string();
    } else if let Some(s) = dict.get(b"name".as_ref()) {
        name = s.string()?.to_string();
    } else {
        name = "".to_string();
    }

    if let Some(x) = dict.get(b"length".as_ref()) {
        length = x.integer()?;
    }

    let mut total_length = 0;
    let mut files = Vec::new();
    if let Some(x) = dict.get(b"files".as_ref()) {
        for f in x.list()? {
            let tf = extract_files(f)?;
            total_length += tf.length;
            files.push(tf);
        }
    }

    if length == 0 {
        length = total_length;
    }

    Ok(Torrent {
        name,
        length,
        files,
    })
}
