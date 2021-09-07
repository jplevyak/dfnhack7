use std::panic;

use crate::{
    codec::{Reader, Writer},
    keyexchange::{client_key_exchange, get_pre_master_secret_secp256k1, EncryptionKeys},
    messages::{
        client_app_data, client_change_cipher_spec, client_close, client_finished, client_hello,
        client_pub_key, create_record, HandshakeMessage, Record, RecordPayloadType, RecordType,
    },
    open::{decrypt, server_finished_verify_data},
    transcript::Transcript,
};

pub enum TlsConnectionState {
    New,
    // ExpectingServerHelloDone,
    // ExpectingServerChangeCipherSpec,
    // ExpectingServerFinished,
    ExpectingServerAppData,
    Finished,
    Error,
}

pub struct TlsConnection {
    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>,

    next_read_index: usize,

    host: String,
    message: Vec<u8>,

    keys: Option<EncryptionKeys>,

    encrypted: bool,

    client_seq: u64,
    server_seq: u64,

    client_private: Vec<u8>,
    client_public: Option<Vec<u8>>,
    client_random: Vec<u8>,

    server_public: Option<Vec<u8>>,
    server_random: Option<Vec<u8>>,

    pre_master_secret: Option<Vec<u8>>,

    transcript: Transcript,

    state: TlsConnectionState,

    result: Vec<u8>,
}

impl TlsConnection {
    pub fn new(host: &str, message: &[u8], client_random: &[u8], client_private: &[u8]) -> Self {
        TlsConnection {
            host: host.to_owned(),
            message: message.to_owned(),
            read_buffer: vec![],
            write_buffer: vec![],
            next_read_index: 0,
            client_random: client_random.to_owned(),
            client_private: client_private.to_owned(),
            client_public: None,
            keys: None,
            encrypted: false,
            client_seq: 0,
            server_seq: 0,
            pre_master_secret: None,
            server_public: None,
            server_random: None,
            transcript: Transcript::new(),
            state: TlsConnectionState::New,
            result: vec![],
        }
    }

    fn hello(&mut self) {
        assert!(matches!(self.state, TlsConnectionState::New));
        let message = &client_hello(&self.host, &self.client_random);
        let record = create_record(0x16, &message);
        self.transcript.add(&message);
        self.write_buffer.extend(record);
        // self.state = TlsConnectionState::ExpectingServerHelloDone;
    }

    pub fn add_incoming_bytes(&mut self, bytes: &[u8]) {
        self.read_buffer.extend_from_slice(bytes);
    }

    fn add_from_seq(content_type: u8, seq: u64) -> Vec<u8> {
        let mut w = Writer::new();
        w.write_u64(seq);
        w.write_u8(content_type);
        w.write_u8(0x03);
        w.write_u8(0x03);
        w.0
    }

    fn send_handshake_record(&mut self, payload: &[u8]) {
        let record = create_record(0x16, payload);
        self.transcript.add(payload);
        self.write_buffer.extend(record);
    }

    fn send_encrypted_handshake_record(&mut self, payload: &[u8], enc_payload: &[u8]) {
        let record = create_record(0x16, enc_payload);
        self.transcript.add(payload);
        self.write_buffer.extend(record);
    }

    pub fn process(&mut self) {
        match &self.state {
            TlsConnectionState::New => {
                self.hello();
                self.state = TlsConnectionState::ExpectingServerAppData;
            }
            _ => {}
        };

        loop {
            let mut reader = Reader::from(&self.read_buffer[self.next_read_index..], 0);

            if !reader.has(1) {
                break;
            }

            let record = Record::from(&mut reader, self.encrypted);

            match record {
                Ok(record) => {
                    self.next_read_index += record.size as usize + 5;
                    match record.payload {
                        RecordPayloadType::ChangeCipherSpec => {
                            println!("Server Change Cipher Spec");
                            self.encrypted = true;
                        }
                        RecordPayloadType::EncryptedHandshake => {
                            println!("Server Finish");

                            if let Some(keys) = &self.keys {
                                match decrypt(
                                    record.raw_payload,
                                    &keys.server_write_iv,
                                    &keys.server_write_key,
                                    &TlsConnection::add_from_seq(0x16, self.server_seq),
                                ) {
                                    Ok(result) => {
                                        let message_hash = self.transcript.get_hash();
                                        let verify_data = server_finished_verify_data(
                                            &message_hash,
                                            &keys.master_secret,
                                        );
                                        assert_eq!(&verify_data, &result[4..]);
                                    }
                                    Err(_) => {
                                        panic!("Error decrypting server finish");
                                    }
                                }

                                let record = create_record(
                                    0x17,
                                    &client_app_data(
                                        &self.message,
                                        self.client_seq,
                                        &keys.client_write_iv,
                                        &keys.client_write_key,
                                    ),
                                );
                                self.client_seq += 1;
                                self.write_buffer.extend_from_slice(&record);
                            } else {
                                panic!("Error: missing decryption keys")
                            }
                        }
                        RecordPayloadType::EncryptedApplicationData => {
                            println!("Client <- Server Application Data");
                            if let Some(keys) = &self.keys {
                                match decrypt(
                                    record.raw_payload,
                                    &keys.server_write_iv,
                                    &keys.server_write_key,
                                    &[
                                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x17, 0x03,
                                        0x03,
                                    ],
                                ) {
                                    Ok(result) => {
                                        self.result = result;

                                        let record = create_record(
                                            0x15,
                                            &client_close(
                                                self.client_seq,
                                                &keys.client_write_iv,
                                                &keys.client_write_key,
                                            ),
                                        );
                                        self.client_seq += 1;
                                        self.write_buffer.extend_from_slice(&record);
                                        self.state = TlsConnectionState::Finished;
                                    }
                                    Err(_) => {
                                        panic!("Error decrypting application data");
                                    }
                                }
                            } else {
                                panic!("Error: missing decryption keys")
                            }
                        }
                        RecordPayloadType::Handshake(m) => match m {
                            HandshakeMessage::ServerCertificate { .. } => {
                                println!("Server Certificate");
                                self.transcript.add(record.raw_payload);
                            }

                            HandshakeMessage::ServerHello { random, .. } => {
                                println!("Server Hello");
                                self.server_random = Some(Vec::from(random));
                                self.transcript.add(record.raw_payload);
                            }
                            HandshakeMessage::ServerKeyExchange { public_key, .. } => {
                                println!("Server Key Exchange");
                                self.server_public = Some(public_key.to_owned());
                                self.transcript.add(record.raw_payload);
                            }
                            HandshakeMessage::ServerHelloDone { .. } => {
                                println!("Server Hello Done");

                                self.transcript.add(record.raw_payload);

                                if let (Some(server_public), Some(server_random)) =
                                    (&self.server_public, &self.server_random)
                                {
                                    let (pre_master_secret, client_public_key) =
                                        get_pre_master_secret_secp256k1(
                                            &self.client_private,
                                            &server_public,
                                        );

                                    let cce = client_key_exchange(
                                        &self.client_random,
                                        &server_random,
                                        &pre_master_secret,
                                    );

                                    self.send_handshake_record(&client_pub_key(&client_public_key));

                                    self.write_buffer
                                        .extend_from_slice(&client_change_cipher_spec());

                                    let (client_finished_msg, enc_client_finished_msg) =
                                        client_finished(
                                            &self.transcript.get_hash(),
                                            &cce.client_write_iv,
                                            &cce.client_write_key,
                                            &cce.master_secret,
                                        );

                                    self.client_seq += 1;

                                    self.send_encrypted_handshake_record(
                                        &client_finished_msg,
                                        &enc_client_finished_msg,
                                    );

                                    self.keys = Some(cce);
                                    self.pre_master_secret = Some(pre_master_secret);
                                    self.client_public = Some(client_public_key);
                                } else {
                                    panic!("Could not get server_public or server_random")
                                }
                            }
                            _ => {}
                        },
                        RecordPayloadType::UnknownPayload(_) => match record.record_type {
                            RecordType::UnknownRecordType(t) => {
                                println!("Unknown record type: {:#02x}", t);
                            }
                            _ => {}
                        },
                    }
                }
                Err(_) => {
                    // data is not fully ready
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::io::prelude::*;
    use std::net::TcpStream;

    use super::TlsConnection;

    #[test]
    fn test_tls_conn() {
        let mut conn = TlsConnection::new(
            "aws.okex.com",
            b"GET /api/spot/v3/instruments/ICP-USDT/ticker HTTP/1.1\nHost: aws.okex.com\nAccept-Encoding: identity\n\n",
            &[
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23, 24, 25, 26, 27, 28, 29, 30, 31,
            ],
            &[
                10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 110, 111, 112, 113, 114, 115, 116, 117,
                118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131,
            ],
        );

        let mut stream = TcpStream::connect("aws.okex.com:443").unwrap();

        let mut buf = [0u8; 1024 * 1024];

        loop {
            conn.process();

            if conn.result.len() > 0 {
                println!("Result:\n{}", String::from_utf8_lossy(&conn.result));
                conn.result.clear();
            }

            if conn.write_buffer.len() > 0 {
                stream.write(&conn.write_buffer).unwrap();
                conn.write_buffer.clear();
            }
            match stream.read(&mut buf) {
                Err(_) => {
                    panic!("Stream read error");
                }
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        // stream done
                        break;
                    }
                    conn.add_incoming_bytes(&buf[0..bytes_read]);
                }
            }
        }
    }
}
