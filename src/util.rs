use rand::RngCore;
use std::ptr::copy;

// infohash length.
const KEY_LENGTH: usize = 20;
// transaction id length.
const TID_LENGTH: usize = 2;

// make random infohash key.
pub fn rand_infohash_key() -> Vec<u8> {
    rand_bytes(KEY_LENGTH)
}

// make random transaction id>
pub fn rand_transation_id() -> Vec<u8> {
    rand_bytes(TID_LENGTH)
}

pub fn neighbor_id(target: &[u8], local: &[u8]) -> Vec<u8> {
    const CLOSENESS: usize = 15;
    let mut id = vec![0; KEY_LENGTH];
    unsafe {
        copy(target.as_ptr(), id.as_mut_ptr(), CLOSENESS);
        copy(
            local.as_ptr().offset(CLOSENESS as isize),
            id.as_mut_ptr().offset(CLOSENESS as isize),
            KEY_LENGTH - CLOSENESS,
        );
    }
    id
}

// make random bytes id.
fn rand_bytes(n: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut ret = vec![0; n];
    rng.fill_bytes(&mut ret);
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rand_bytes() {
        let b1 = rand_bytes(KEY_LENGTH);
        assert_eq!(b1.len(), KEY_LENGTH);
        let b2 = rand_bytes(KEY_LENGTH);
        assert_eq!(b2.len(), KEY_LENGTH);
        assert_ne!(b1, b2);
    }

    #[test]
    fn test_neighbor_id() {
        todo!();
    }
}
