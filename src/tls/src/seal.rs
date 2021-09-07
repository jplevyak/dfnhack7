use aes_gcm::aead::{Aead, NewAead, Payload};
use aes_gcm::{Aes128Gcm, Key, Nonce};

use crate::{codec::Writer, prf::prf, utils::concat};

pub fn client_finished_verify_data(message_hash: &[u8], master_secret: &[u8]) -> Vec<u8> {
    let data = prf(master_secret, b"client finished", message_hash, 12);

    assert_eq!(data.len(), 12);
    data
}

pub fn encrypt(
    message: &[u8],
    seq_num: u64,
    content_type: u8,
    client_write_iv: &[u8],
    client_write_key: &[u8],
) -> Vec<u8> {
    let nonce = b"oO\x17V\xdd>\xae\xa1";

    let mut additional_bytes = Writer::new();
    additional_bytes.write_u64(seq_num);
    additional_bytes.append(&[content_type, 3, 3]);
    additional_bytes.write_u16(message.len() as u16);

    let cipher = Aes128Gcm::new(Key::from_slice(client_write_key));

    let iv_nonce = concat(client_write_iv, nonce);
    let final_nonce = Nonce::from_slice(&iv_nonce); // 96-bits; unique per message

    let encrypted = Vec::from(
        cipher
            .encrypt(
                final_nonce,
                Payload {
                    msg: message,
                    aad: &additional_bytes.0,
                },
            )
            .unwrap(),
    );

    concat(nonce, &encrypted)
}

pub fn encrypt_client_finish(
    verify_data: &[u8],
    client_write_iv: &[u8],
    client_write_key: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    let seq_num = 0u64;

    let mut hs_message = Writer::new();
    hs_message.write_u8(0x14);
    hs_message.append_with_size_u24(verify_data);

    let result = encrypt(
        &hs_message.0,
        seq_num,
        0x16u8,
        client_write_iv,
        client_write_key,
    );

    (hs_message.0, result)
}

#[cfg(test)]
mod tests {

    use super::{client_finished_verify_data, encrypt_client_finish};

    #[test]
    fn test_verify_data() {
        let hash =
            b"|^\xb7~\xb1\xd2D\xe6m\xf5eO*\x8a?\x14g\xfb\xe4\xa6)\xf8\x94P\xe9\xcbv\xc8\xde\xe4Go";
        let master_secret = b"\x0c\x97j\xc2#2\x10\x14@\xbb\xc3?)\xf7>\xbc\xe2\x99\xc9Yf~\x17\xfdo\x99\xca;!2\x8fO\x148\x89sb\xcb\x0c\x12\xafr\xf1M\x961~+";

        let expected_verify_data = b"\x0c\x84:\xf5\xe2y\xbb\xc9\x9c\x91Q\x1c";
        let verify_data = client_finished_verify_data(hash, master_secret);

        assert_eq!(verify_data.as_slice(), expected_verify_data);
    }

    #[test]
    fn test_encrypt_client_finish() {
        let verify_data = b"\x0c\x84:\xf5\xe2y\xbb\xc9\x9c\x91Q\x1c";
        let client_write_iv = b"Li\x9d\x9f";
        let client_write_key = b"5\x1c3\x0e\xdb\xd7\xb3\xc0\x1e1\xd1f\x95\xc2\xbbd";

        let result = encrypt_client_finish(verify_data, client_write_iv, client_write_key);

        let expected_result = b"oO\x17V\xdd>\xae\xa1f\x11\xb6\xf1*\xcb;\xec\xcf\xde\xba%R\x9a\xb8\xde\xe0X\xf1\xcc\x99\x1f\xcc2[\xc2AK\xeb\xd8\xf6\xca";

        assert_eq!(result.1.as_slice(), expected_result);
    }
}
