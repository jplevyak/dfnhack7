use aes_gcm::aead::{Aead, NewAead, Payload};
use aes_gcm::{Aes128Gcm, Error, Key, Nonce};

use crate::codec::Writer;
use crate::prf::prf;
use crate::utils::concat;

pub fn server_finished_verify_data(message_hash: &[u8], master_secret: &[u8]) -> Vec<u8> {
    let data = prf(master_secret, b"server finished", message_hash, 12);

    assert_eq!(data.len(), 12);
    data
}

pub fn decrypt(
    message: &[u8],
    server_write_iv: &[u8],
    server_write_key: &[u8],
    add: &[u8],
) -> Result<Vec<u8>, Error> {
    let cipher = Aes128Gcm::new(Key::from_slice(server_write_key));

    assert_eq!(server_write_iv.len(), 4);

    let nonce = &message[0..8];
    let iv_nonce = concat(server_write_iv, nonce);
    let final_nonce = Nonce::from_slice(iv_nonce.as_slice());

    let mut final_add = Writer::new();
    final_add.append(add);
    final_add.write_u16(message.len() as u16 - 8 - 16);

    cipher.decrypt(
        final_nonce,
        Payload {
            msg: &message[8..message.len()],
            aad: final_add.0.as_slice(),
        },
    )
}

#[cfg(test)]
mod tests {

    use super::decrypt;

    #[test]
    fn test_decrypt() {
        let key: &[u8] = &[
            0x72, 0xA7, 0x3E, 0x81, 0x90, 0x55, 0xEE, 0xE6, 0x33, 0xA7, 0x60, 0xF3, 0x26, 0x05,
            0xBD, 0xDC,
        ];
        let iv: &[u8] = &[0xD1, 0xED, 0x8B, 0x88];
        let encrypted_message: &[u8] = &[
            0x3B, 0x63, 0x50, 0xF1, 0x9C, 0x89, 0xAC, 0x3D, 0x7F, 0xC1, 0x54, 0xC6, 0xD9, 0x12,
            0xB4, 0xE7, 0x5E, 0xCA, 0x1E, 0xCA, 0x85, 0xBE, 0x1D, 0xA6, 0x95, 0xA1, 0xAA, 0xD5,
            0xFC, 0xB9, 0xB0, 0x06, 0xA3, 0x9E, 0x49, 0x53, 0x4C, 0xF2, 0x8D, 0x39,
        ];
        let add: &[u8] = &[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x16, 0x03, 0x03,
        ];

        let expected_result = &[
            0x14, 0x00, 0x00, 0x0C, 0x11, 0xB4, 0x69, 0x53, 0x85, 0x84, 0x55, 0x56, 0xD8, 0xDE,
            0xEB, 0xD8,
        ];

        let result = decrypt(encrypted_message, iv, key, add);

        match result {
            Ok(result) => {
                assert_eq!(expected_result, result.as_slice());
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false)
            }
        }
    }
}
