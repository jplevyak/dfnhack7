use std::iter::FromIterator;

use crate::messages::HmacSha256;
use hmac::{Mac, NewMac};

fn a(secret: &[u8], n: usize, seed: &[u8]) -> Vec<u8> {
    match n {
        0 => seed.into(),
        _ => {
            let mut mac = HmacSha256::new_from_slice(secret).unwrap();
            mac.update(a(secret, n - 1, seed).as_slice());
            Vec::from_iter(mac.finalize().into_bytes())
        }
    }
}

pub fn p(secret: &[u8], seed: &[u8], output_length: usize) -> Vec<u8> {
    let mut i = 1usize;
    let mut result = vec![0u8; 0];

    while result.len() < output_length {
        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        mac.update(a(secret, i, seed).as_slice());
        mac.update(seed);
        result.extend(mac.finalize().into_bytes());
        i += 1;
    }

    result.truncate(output_length);
    result
}

pub fn prf(secret: &[u8], label: &[u8], seed: &[u8], output_length: usize) -> Vec<u8> {
    let mut input = Vec::from(label);
    input.extend(seed);
    p(secret, input.as_slice(), output_length)
}

#[cfg(test)]
mod tests {
    use super::prf;

    #[test]
    fn test_master_secret() {
        let pre_master_secret = b"O\xf7\xe8\xc2!\xce\x08\xb1\xa2\xcb\xad\xc1]\xf8\xee\xde\x88\xbe\x9a\x82H\xf6\xe4\x184\xc8\x9b\x9f\x07,E\xc3";
        // seed = client_random + server_random
        let seed = b"a.\x82\xd4\xa8]_\x91\xf9\xd18\xae\xe2\xb8\xf9BS\xd1\x07\xafjH\x99@\t\xcb\x031\x90f\xf2\x11\xf7\x1c\x1bdl\xe8!\x92`\xd5xnd\x7f\x83\xc0\xaa\xce>*LZ\xa2w\x82\x97e\xd8\xbaZS\x04";
        let expected_master_secret= b"\x0c\x97j\xc2#2\x10\x14@\xbb\xc3?)\xf7>\xbc\xe2\x99\xc9Yf~\x17\xfdo\x99\xca;!2\x8fO\x148\x89sb\xcb\x0c\x12\xafr\xf1M\x961~+";

        let master_secret = prf(pre_master_secret, b"master secret", seed, 48);
        assert_eq!(master_secret.as_slice(), expected_master_secret);
    }

    #[test]
    fn test_key_block() {
        let master_secret= b"\x0c\x97j\xc2#2\x10\x14@\xbb\xc3?)\xf7>\xbc\xe2\x99\xc9Yf~\x17\xfdo\x99\xca;!2\x8fO\x148\x89sb\xcb\x0c\x12\xafr\xf1M\x961~+";
        let seed = b"\xf7\x1c\x1bdl\xe8!\x92`\xd5xnd\x7f\x83\xc0\xaa\xce>*LZ\xa2w\x82\x97e\xd8\xbaZS\x04a.\x82\xd4\xa8]_\x91\xf9\xd18\xae\xe2\xb8\xf9BS\xd1\x07\xafjH\x99@\t\xcb\x031\x90f\xf2\x11";
        let expected_kb = b"5\x1c3\x0e\xdb\xd7\xb3\xc0\x1e1\xd1f\x95\xc2\xbbd\x10\xa4\x04Ff6_\xd8\x01\xd1l\x97\x10SI\xc9Li\x9d\x9f\x04\x13\x7f\x043T*\ns\x90\xccG\\\xf5s2\xb3\xbc\xe7\x19\xde\x10\xa0e\x940HH\xcc\xd56\x95_\x89\x16\xbd`\xaf~r\\M\x07\xf7\xd8A\x9fI\x14P\x8a$\xa89\xb2i\xd4\xfa[\x9c<(\xb6k\xc0{\xad\xf9{[7\x07\x12\"\x03H\x08E\xaa\x1dj\x97\xa7\xa7=I\x1e6\x80\x15\x93\x0c\xb2\x93\xb1f^|\xbbp\xc7\xf5\xae\xa8\xb2u\xbdDi\x07\xaa$\xdc\xec\xa5i \xb2 \xed\x7f]\xd9\x95d\xf6\xbfO\xc4\x7f\xb0L\r9b\x1e\x01i\x87\xf5a\x1e.\x16-r\x01\x8a\x97\x87\xfc\xab\x83\x02FdY\xa4\xa7\xaaw\xce{\x10";

        let kb = prf(master_secret, b"key expansion", seed, 200);

        assert_eq!(kb.as_slice(), expected_kb);
    }
}
