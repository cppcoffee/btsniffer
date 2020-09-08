use crate::{Error, Result, Value};

use std::collections::HashMap;
use std::io::{BufRead, Read};
use std::str;

struct Decoder<'a> {
    data: &'a [u8],
}

impl<'a> Decoder<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    // skip one byte.
    fn skip_byte(&mut self) -> Result<()> {
        let mut tmp = [0; 1];
        self.data.read(&mut tmp)?;
        Ok(())
    }

    // Strings are length-prefixed base ten followed by a colon and the string.
    // For example 4:spam corresponds to 'spam'.
    fn read_byte_string(&mut self) -> Result<Value> {
        let mut buf = Vec::new();
        self.data.read_until(b':', &mut buf)?;

        let n = str::from_utf8(&buf[..buf.len() - 1])?.parse::<i64>()?;

        buf.clear();
        buf.resize_with(n as usize, Default::default);

        self.data.read_exact(&mut buf)?;

        Ok(Value::ByteString(buf))
    }

    // Integers are represented by an 'i' followed by the number in base 10
    // followed by an 'e'. For example i3e corresponds to 3 and i-3e corresponds
    // to -3. Integers have no size limitation. i-0e is invalid. All encodings with
    // a leading zero, such as i03e, are invalid, other than i0e, which of course
    // corresponds to 0.
    fn read_integer(&mut self) -> Result<Value> {
        self.skip_byte()?;

        let mut buf = Vec::new();
        self.data.read_until(b'e', &mut buf)?;

        let s = str::from_utf8(&buf[..buf.len() - 1])?;
        if s.starts_with("-0") || (s.len() > 1 && s.starts_with("0")) {
            return Err(Error::Other(format!("invalid integer '{}'", s)));
        }

        Ok(Value::Integer(s.parse::<i64>()?))
    }

    // Lists are encoded as an 'l' followed by their elements (also bencoded)
    // followed by an 'e'. For example l4:spam4:eggse corresponds to ['spam', 'eggs'].
    fn read_list(&mut self) -> Result<Value> {
        self.skip_byte()?;

        let mut res = Vec::new();
        loop {
            let mut p = self.data.iter().peekable();
            match p.peek() {
                Some(b'e') => {
                    self.skip_byte()?;
                    break;
                }
                Some(_) => res.push(self.read_value()?),
                None => {
                    return Err(Error::Other("eof stream".to_string()));
                }
            }
        }
        Ok(Value::List(res))
    }

    // Dictionaries are encoded as a 'd' followed by a list of alternating keys
    // and their corresponding values followed by an 'e'.
    // For example, d3:cow3:moo4:spam4:eggse corresponds to {'cow': 'moo', 'spam': 'eggs'}
    // and d4:spaml1:a1:bee corresponds to {'spam': ['a', 'b']}. Keys must be strings
    // and appear in sorted order (sorted as raw strings, not alphanumerics).
    fn read_dict(&mut self) -> Result<Value> {
        self.skip_byte()?;

        let mut res = HashMap::new();
        loop {
            let mut p = self.data.iter().peekable();
            let ch = p.peek();
            if ch.is_none() || ch == Some(&&b'e') {
                self.skip_byte()?;
                break;
            }

            let key = match self.read_value()? {
                Value::ByteString(inner) => inner,
                _ => unreachable!(),
            };

            let val = self.read_value()?;
            res.insert(key, val);
        }
        Ok(Value::Dict(res))
    }

    fn read_value(&mut self) -> Result<Value> {
        let mut p = self.data.iter().peekable();

        match p.peek() {
            Some(b'i') => self.read_integer(),
            Some(b'l') => self.read_list(),
            Some(b'd') => self.read_dict(),
            Some(_) => self.read_byte_string(),
            None => Err(Error::Other("eof stream".to_string())),
        }
    }
}

pub fn from_bytes(v: &[u8]) -> Result<Value> {
    let mut decoder = Decoder::new(v);
    decoder.read_value()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bencode::map;

    #[test]
    fn test_decode_byte_string() {
        map!(b"2:ab".as_ref() => b"ab".as_ref(),
             b"1:c".as_ref() => b"c".as_ref(),
             b"0:".as_ref() => b"".as_ref())
        .iter()
        .for_each(|(k, v)| {
            let mut de = Decoder::new(k);
            let val = de.read_byte_string();
            assert!(val.is_ok());
            assert_eq!(Value::from(*v), val.unwrap());
        });

        // invalid byte string.
        [
            b"3:ab".as_ref(),
            b"3abc".as_ref(),
            b":abc".as_ref(),
            b"1:".as_ref(),
        ]
        .iter()
        .for_each(|x| {
            let mut de = Decoder::new(x.as_ref());
            let val = de.read_byte_string();
            assert!(val.is_err());
        });
    }

    #[test]
    fn test_decode_integer() {
        map!(b"i0e".as_ref() => 0,
             b"i1e".as_ref() => 1,
             b"i66e".as_ref() => 66,
             b"i-1e".as_ref() => -1,
             b"i-22e".as_ref() => -22)
        .iter()
        .for_each(|(k, v)| {
            let mut de = Decoder::new(k.as_ref());
            let val = de.read_integer();
            assert!(val.is_ok());
            assert_eq!(Value::from(*v), val.unwrap());
        });

        // invalid integer.
        [b"i-0e".as_ref(), b"i03e".as_ref(), b"i002e".as_ref()]
            .iter()
            .for_each(|x| {
                let mut de = Decoder::new(x.as_ref());
                let val = de.read_integer();
                assert!(val.is_err());
            });
    }

    #[test]
    fn test_decode_list() {
        map!(b"l4:spam4:eggse".as_ref() => [b"spam", b"eggs"].as_ref(),
             b"le".as_ref() => [].as_ref())
        .iter()
        .for_each(|(k, v)| {
            let mut de = Decoder::new(k);
            let val = de.read_list();
            assert!(val.is_ok());

            let mut res = Vec::new();
            for x in v.iter() {
                res.push(Value::from(x.as_ref()));
            }
            assert_eq!(Value::from(res), val.unwrap());
        });

        // invalid list.
        [b"l1:a".as_ref(), b"l".as_ref()].iter().for_each(|x| {
            let mut de = Decoder::new(x);
            let val = de.read_list();
            assert!(val.is_err());
        });
    }

    #[test]
    fn test_decode_dict() {
        // d3:cow3:moo4:spam4:eggse => {'cow': 'moo', 'spam': 'eggs'}
        // d4:spaml1:a1:bee => {'spam': ['a', 'b']}
        // de => {}
        map!(
          b"d4:spaml1:a1:bee".as_ref() =>
            map!(b"spam".to_vec() => Value::List([Value::from(b"a".as_ref()),
                                       Value::from(b"b".as_ref())].to_vec())),
          b"d3:cow3:moo4:spam4:eggse".as_ref() =>
            map!(b"cow".to_vec() => Value::from(b"moo".to_vec()),
                 b"spam".to_vec() => Value::from(b"eggs".to_vec())),
          b"be" => HashMap::new(),
          b"d1:td1:r2:abee".as_ref() =>
            map!(b"t".to_vec() => Value::from(
              map!(b"r".to_vec() => Value::from(b"ab".to_vec()))))
        )
        .iter()
        .for_each(|(k, v)| {
            let mut de = Decoder::new(k);
            let dict = de.read_dict();
            assert!(dict.is_ok());
            assert_eq!(Value::from(v.clone()), dict.unwrap());
        })
    }

    #[test]
    fn test_decode_from_bytes() {
        // d1:eli201e23:A Generic Error Ocurrede1:t2:aa1:y1:ee => {"t":"aa", "y":"e", "e":[201, "A Generic Error Ocurred"]}
        // d1:rd2:id20:mnopqrstuvwxyz123456e1:t2:aa1:y1:re => {"t":"aa", "y":"r", "r": {"id":"mnopqrstuvwxyz123456"}}
        map!(
          b"d1:eli201e23:A Generic Error Ocurrede1:t2:aa1:y1:ee".as_ref() =>
            map!(b"t".to_vec() => Value::from(b"aa".as_ref()),
                 b"y".to_vec() => Value::from(b"e".as_ref()),
                 b"e".to_vec() => Value::List([Value::from(201), Value::from(b"A Generic Error Ocurred".as_ref())].to_vec())),
          b"d1:rd2:id4:moone1:t2:aa1:y1:re".as_ref() =>
            map!(b"t".to_vec() => Value::from(b"aa".as_ref()),
                 b"y".to_vec() => Value::from(b"r".as_ref()),
                 b"r".to_vec() => Value::from(map!(b"id".to_vec() => Value::from(b"moon".as_ref())))))
        .iter()
        .for_each(|(k, v)| {
            let res = from_bytes(k);
            assert!(res.is_ok());
            assert_eq!(Value::from(v.clone()), res.unwrap());
        });
    }
}
