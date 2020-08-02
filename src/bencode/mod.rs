pub mod error;
pub use error::{Error, Result};

pub mod value;
pub use value::Value;

pub mod decoder;
pub use decoder::from_bytes;

pub mod encoder;
pub use encoder::to_bytes;

#[macro_export]
macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);
pub use map;
