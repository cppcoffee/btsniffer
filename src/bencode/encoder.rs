use crate::bencode::{Result, Value};
use std::collections::HashMap;
use std::io::Write;

struct Encoder {
    buf: Vec<u8>,
}

impl Encoder {
    fn new() -> Self {
        Self { buf: Vec::new() }
    }

    fn write_byte_string(&mut self, v: &[u8]) -> Result<()> {
        self.buf.write(v.len().to_string().as_bytes())?;
        self.buf.write(b":")?;
        self.buf.write(v)?;
        Ok(())
    }

    fn write_integer(&mut self, v: i64) -> Result<()> {
        self.buf.write(b"i")?;
        self.buf.write(v.to_string().as_bytes())?;
        self.buf.write(b"e")?;
        Ok(())
    }

    fn write_list(&mut self, l: &[Value]) -> Result<()> {
        self.buf.write(b"l")?;
        for v in l.iter() {
            self.write_value(&v)?;
        }
        self.buf.write(b"e")?;
        Ok(())
    }

    fn write_dict(&mut self, dict: &HashMap<Vec<u8>, Value>) -> Result<()> {
        self.buf.write(b"d")?;
        for (key, val) in dict.iter() {
            self.write_byte_string(key)?;
            self.write_value(val)?;
        }
        self.buf.write(b"e")?;
        Ok(())
    }

    fn write_value(&mut self, value: &Value) -> Result<()> {
        match value {
            Value::ByteString(ref v) => self.write_byte_string(v),
            Value::Integer(ref v) => self.write_integer(*v),
            Value::List(ref v) => self.write_list(v),
            Value::Dict(ref v) => self.write_dict(v),
        }
    }

    fn buffer(&mut self) -> &[u8] {
        &self.buf
    }
}

// Value encode convert bytes.
pub fn to_bytes(value: &Value) -> Result<Vec<u8>> {
    let mut encoder = Encoder::new();
    encoder.write_value(value)?;
    Ok(encoder.buffer().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bencode::map;

    #[test]
    fn test_encode_byte_string() {
        map!(b"ab".as_ref() => b"2:ab".as_ref(),
             b"".as_ref() => b"0:".as_ref(),
             b"123456".as_ref() => b"6:123456".as_ref())
        .iter()
        .for_each(|(k, v)| {
            let mut ser = Encoder::new();
            let res = ser.write_byte_string(k);
            assert!(res.is_ok());
            assert_eq!(*v, ser.buffer());
        });
    }

    #[test]
    fn test_encode_integer() {
        map!(0 => b"i0e".as_ref(),
             -1 => b"i-1e",
             999 => b"i999e")
        .iter()
        .for_each(|(k, v)| {
            let mut ser = Encoder::new();
            let res = ser.write_integer(*k);
            assert!(res.is_ok());
            assert_eq!(*v, ser.buffer());
        });
    }

    #[test]
    fn test_encode_list() {
        map!(b"le".as_ref() => vec![],
          b"li0ei1ei2ee".as_ref() => vec![Value::from(0), Value::from(1), Value::from(2)],
          b"li-123e3:abce".as_ref() => vec![Value::from(-123), Value::from(b"abc".as_ref())],
          b"l3:abci10ee".as_ref() => vec![Value::from(b"abc".as_ref()), Value::from(10)])
        .iter()
        .for_each(|(k, v)| {
            let mut ser = Encoder::new();
            let res = ser.write_list(v);
            assert!(res.is_ok());
            assert_eq!(*k, ser.buffer());
        });
    }

    #[test]
    fn test_encode_dict() {
        map!(
          b"d2:idi123ee".to_vec() => map!(b"id".to_vec() => Value::from(123)),
          b"d4:pingli1e2:abee".to_vec() =>
            map!(b"ping".to_vec() => Value::from(vec![Value::from(1), Value::from(b"ab".to_vec())])))
        .iter()
        .for_each(|(k, v)| {
            let mut ser = Encoder::new();
            let res = ser.write_dict(v);
            assert!(res.is_ok());
            assert_eq!(*k, ser.buffer());
        });
    }

    #[test]
    #[ignore]
    fn test_encode_to_bytes() {
        map!(
          b"d1:ad2:id4:abcde1:q4:ping1:t2:aa1:y1:qe".to_vec() => Value::from(
            map!(b"a".to_vec() => Value::from(map!(b"id".to_vec() => Value::from(b"abcd".to_vec()))),
                 b"q".to_vec() => Value::from(b"ping".to_vec()),
                 b"t".to_vec() => Value::from(b"aa".to_vec()),
                 b"y".to_vec() => Value::from(b"q".to_vec()))))
        .iter()
        .for_each(|(k, v)| {
            let buf = to_bytes(v);
            assert!(buf.is_ok());
            assert_eq!(*k, buf.unwrap());
        });
    }
}
