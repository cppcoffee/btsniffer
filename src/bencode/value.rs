use crate::bencode::{Error, Result};
use std::collections::HashMap;

#[derive(PartialEq, Clone, Debug)]
pub enum Value {
    ByteString(Vec<u8>),
    Integer(i64),
    List(Vec<Value>),
    Dict(HashMap<Vec<u8>, Value>),
}

impl Value {
    pub fn bytes(&self) -> Result<&[u8]> {
        match self {
            Value::ByteString(x) => Ok(&x),
            _ => Err(Error::NotByteStringType),
        }
    }

    pub fn string(&self) -> Result<&str> {
        match self {
            Value::ByteString(x) => Ok(unsafe { std::str::from_utf8_unchecked(x) }),
            _ => Err(Error::NotByteStringType),
        }
    }

    pub fn dict(&self) -> Result<&HashMap<Vec<u8>, Value>> {
        match self {
            Value::Dict(ref m) => Ok(m),
            _ => Err(Error::NotDictType),
        }
    }

    pub fn list(&self) -> Result<&Vec<Value>> {
        match self {
            Value::List(ref x) => Ok(x),
            _ => Err(Error::NotListType),
        }
    }

    pub fn integer(&self) -> Result<i64> {
        match self {
            Value::Integer(n) => Ok(*n),
            _ => Err(Error::NotIntegerType),
        }
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::ByteString(v)
    }
}

impl From<&[u8]> for Value {
    fn from(v: &[u8]) -> Self {
        Value::ByteString(Vec::from(v))
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::ByteString(v.as_bytes().to_vec())
    }
}

impl From<i8> for Value {
    fn from(n: i8) -> Self {
        Value::Integer(n.into())
    }
}

impl From<i16> for Value {
    fn from(n: i16) -> Self {
        Value::Integer(n.into())
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Integer(n.into())
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Integer(n)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::List(v)
    }
}

impl From<&[Value]> for Value {
    fn from(v: &[Value]) -> Self {
        Value::List(v.to_vec())
    }
}

impl From<HashMap<Vec<u8>, Value>> for Value {
    fn from(v: HashMap<Vec<u8>, Value>) -> Self {
        Value::Dict(v)
    }
}
