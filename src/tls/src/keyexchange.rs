use k256::{ecdh::EphemeralSecret, EncodedPoint, PublicKey};

use crate::{prf::prf, random::IcRandom6, utils::concat};

pub fn get_pre_master_secret_secp256k1(
    client_private_key: &[u8],
    server_pub_key: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    let mut rng = IcRandom6::new(client_private_key);

    let client_private = EphemeralSecret::random(&mut rng);
    let client_public = EncodedPoint::from(client_private.public_key());

    // println!(
    //     "client public key = {:?}",
    //     client_public.as_bytes().to_owned()
    // );

    let bob_public = PublicKey::from_sec1_bytes(server_pub_key).unwrap();

    let shared_secret = client_private.diffie_hellman(&bob_public);

    let mut pre_master_secret: Vec<u8> = vec![];
    pre_master_secret.extend(shared_secret.as_bytes());
    (pre_master_secret, client_public.as_bytes().to_owned())
}

#[derive(Debug)]
pub struct EncryptionKeys {
    pub client_write_mac_key: Vec<u8>,
    pub server_write_mac_key: Vec<u8>,
    pub client_write_key: Vec<u8>,
    pub server_write_key: Vec<u8>,
    pub client_write_iv: Vec<u8>,
    pub server_write_iv: Vec<u8>,
    pub master_secret: Vec<u8>,
}

pub fn client_key_exchange(
    client_hello_random: &[u8],
    server_hello_random: &[u8],
    pre_master_secret: &[u8],
) -> EncryptionKeys {
    let seed = concat(client_hello_random, server_hello_random);
    let master_secret = prf(pre_master_secret, b"master secret", seed.as_slice(), 48);

    let seed = concat(server_hello_random, client_hello_random);
    let mut kb = prf(
        master_secret.as_slice(),
        b"key expansion",
        seed.as_slice(),
        200,
    );

    let hash_size = 0;
    let key_size = 16;
    let iv_size = 4;

    EncryptionKeys {
        master_secret,
        client_write_mac_key: kb.drain(0..hash_size).collect(),
        server_write_mac_key: kb.drain(0..hash_size).collect(),
        client_write_key: kb.drain(0..key_size).collect(),
        server_write_key: kb.drain(0..key_size).collect(),
        client_write_iv: kb.drain(0..iv_size).collect(),
        server_write_iv: kb.drain(0..iv_size).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::client_key_exchange;

    #[test]
    fn test_key_exchange() {
        let pre_master_secret = b"O\xf7\xe8\xc2!\xce\x08\xb1\xa2\xcb\xad\xc1]\xf8\xee\xde\x88\xbe\x9a\x82H\xf6\xe4\x184\xc8\x9b\x9f\x07,E\xc3";
        let client_random = b"a.\x82\xd4\xa8]_\x91\xf9\xd18\xae\xe2\xb8\xf9BS\xd1\x07\xafjH\x99@\t\xcb\x031\x90f\xf2\x11";
        let server_random =
            b"\xf7\x1c\x1bdl\xe8!\x92`\xd5xnd\x7f\x83\xc0\xaa\xce>*LZ\xa2w\x82\x97e\xd8\xbaZS\x04";

        let expected_master_secret= b"\x0c\x97j\xc2#2\x10\x14@\xbb\xc3?)\xf7>\xbc\xe2\x99\xc9Yf~\x17\xfdo\x99\xca;!2\x8fO\x148\x89sb\xcb\x0c\x12\xafr\xf1M\x961~+";

        let keys = client_key_exchange(client_random, server_random, pre_master_secret);

        assert_eq!(keys.master_secret.as_slice(), expected_master_secret);

        assert_eq!(keys.client_write_mac_key.as_slice(), b"");
        assert_eq!(keys.server_write_mac_key.as_slice(), b"");
        assert_eq!(
            keys.client_write_key.as_slice(),
            b"5\x1c3\x0e\xdb\xd7\xb3\xc0\x1e1\xd1f\x95\xc2\xbbd"
        );
        assert_eq!(
            keys.server_write_key.as_slice(),
            b"\x10\xa4\x04Ff6_\xd8\x01\xd1l\x97\x10SI\xc9"
        );
        assert_eq!(keys.client_write_iv.as_slice(), b"Li\x9d\x9f");
        assert_eq!(keys.server_write_iv.as_slice(), b"\x04\x13\x7f\x04");
    }
}
