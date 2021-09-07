use crate::codec::{Reader, ReaderError, Writer};
use crate::seal::{client_finished_verify_data, encrypt, encrypt_client_finish};
use hmac::Hmac;
use sha2::Sha256;

pub type HmacSha256 = Hmac<Sha256>;

pub fn create_record(content_type: u8, payload: &[u8]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_u8(content_type);
    w.write_u8(0x03);
    w.write_u8(0x03);
    w.append_with_size_u16(payload);
    w.0
}

pub fn client_hello(host: &str, client_random: &[u8]) -> Vec<u8> {
    let mut w = Writer::new();
    w.append(&[0x01]);
    w.append_with_size_u24(
        {
            let mut w = Writer::new();

            w.append(&[0x03, 0x03]);

            w.append(client_random);
            w.write_u8(0x00);
            w.append_with_size_u16(&[
                0xc0,
                0x2f, // https://ciphersuite.info/cs/TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256/
            ]);
            w.append(&[0x01, 0x00]);

            // EXTENSIONS
            w.append_with_size_u16(
                {
                    let mut w = Writer::new();

                    // server name
                    w.append(&[0, 0]);
                    w.append_with_size_u16(
                        {
                            let mut w = Writer::new();
                            w.append_with_size_u16(
                                {
                                    let mut w = Writer::new();
                                    w.write_u8(0);
                                    w.append_string_size_u16(host);
                                    w.0
                                }
                                .as_slice(),
                            );
                            w.0
                        }
                        .as_slice(),
                    );

                    w.append(&[0x00, 0x05, 0x00, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00]);
                    w.append(&[
                        0x00, 0x0a, // supported curves
                        0x00, 0x04, 0x00, 0x02, 0x00, 0x16, // only support secp256k1
                    ]);
                    w.append(&[0x00, 0x0b, 0x00, 0x02, 0x01, 0x00]);
                    w.append(&[
                        // signature algos
                        0x00, 0x0d, 0x00, 0x12, 0x00, 0x10, 0x04, 0x01, 0x04, 0x03, 0x05, 0x01,
                        0x05, 0x03, 0x06, 0x01, 0x06, 0x03, 0x02, 0x01, 0x02, 0x03,
                    ]);
                    w.append(&[0xff, 0x01, 0x00, 0x01, 0x00]);
                    w.append(&[0x00, 0x12, 0x00, 0x00]);

                    w.0
                }
                .as_slice(),
            );

            w.0
        }
        .as_slice(),
    );

    w.0
}

pub fn client_pub_key(pub_key: &[u8]) -> Vec<u8> {
    let mut w = Writer::new();
    w.append(&[0x10]);

    w.append_with_size_u24(
        {
            let mut w = Writer::new();
            w.append_with_size_u8(pub_key);
            w.0
        }
        .as_slice(),
    );

    w.0
}

pub fn client_change_cipher_spec() -> Vec<u8> {
    vec![0x14, 0x03, 0x03, 0x00, 0x01, 0x01]
}

pub fn client_finished(
    messages_hash: &[u8],
    client_write_iv: &[u8],
    client_write_key: &[u8],
    master_secret: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    let verify_data = client_finished_verify_data(&messages_hash, master_secret);
    encrypt_client_finish(&verify_data, client_write_iv, client_write_key)
}

pub fn client_app_data(
    message: &[u8],
    seq_num: u64,
    client_write_iv: &[u8],
    client_write_key: &[u8],
) -> Vec<u8> {
    encrypt(message, seq_num, 0x17, client_write_iv, client_write_key)
}

pub fn client_close(seq_num: u64, client_write_iv: &[u8], client_write_key: &[u8]) -> Vec<u8> {
    let message = &[0x01, 0x00];
    encrypt(message, seq_num, 0x15, client_write_iv, client_write_key)
}

pub enum RecordError {
    InsufficentData,
    // UnknownRecordType { message_type: u8 },
}

#[derive(Clone, Debug)]
pub enum RecordType {
    Handshake,
    ChangeCipherSpec,
    EncryptedApplicationData,
    UnknownRecordType(u8),
}

impl From<u8> for RecordType {
    fn from(v: u8) -> Self {
        match v {
            x if x == 0x16u8 => RecordType::Handshake,
            x if x == 0x17u8 => RecordType::EncryptedApplicationData,
            x if x == 0x14u8 => RecordType::ChangeCipherSpec,
            x => RecordType::UnknownRecordType(x),
        }
    }
}

#[derive(Clone, Debug)]
pub enum HandshakeMessageType {
    ServerHello,
    ServerCertificate,
    ServerKeyExchange,
    ServerHelloDone,
    UnknownMessageType(u8),
}

impl From<u8> for HandshakeMessageType {
    fn from(v: u8) -> Self {
        match v {
            x if x == 0x02 => HandshakeMessageType::ServerHello,
            x if x == 0x0b => HandshakeMessageType::ServerCertificate,
            x if x == 0x0c => HandshakeMessageType::ServerKeyExchange,
            x if x == 0x0e => HandshakeMessageType::ServerHelloDone,
            x => HandshakeMessageType::UnknownMessageType(x),
        }
    }
}

#[derive(Clone, Debug)]
pub enum RecordPayloadType<'a> {
    Handshake(HandshakeMessage<'a>),
    EncryptedHandshake,
    EncryptedApplicationData,
    ChangeCipherSpec,
    UnknownPayload(&'a [u8]),
}

#[derive(Clone, Debug)]
pub struct Record<'a> {
    pub record_type: RecordType,
    pub protocol_version: u16,
    pub size: u16,
    pub raw_payload: &'a [u8],
    pub payload: RecordPayloadType<'a>,
}

impl<'a> Record<'a> {
    pub fn from(reader: &'a mut Reader, encrypted: bool) -> Result<Self, ReaderError> {
        let record_type: RecordType = reader.read_u8()?.into();

        match record_type {
            RecordType::ChangeCipherSpec => {
                let protocol_version = reader.read_u16()?;
                let size = reader.read_u16()?;
                let raw_payload = reader.read_n(size.into())?;

                Ok(Record {
                    payload: RecordPayloadType::ChangeCipherSpec,
                    size,
                    record_type: RecordType::ChangeCipherSpec,
                    protocol_version,
                    raw_payload,
                })
            }
            RecordType::EncryptedApplicationData => {
                assert!(encrypted);
                let protocol_version = reader.read_u16()?;
                let size = reader.read_u16()?;
                let raw_payload = reader.read_n(size.into())?;

                Ok(Record {
                    payload: RecordPayloadType::EncryptedApplicationData,
                    size,
                    record_type: RecordType::EncryptedApplicationData,
                    protocol_version,
                    raw_payload,
                })
            }
            RecordType::Handshake => {
                let protocol_version = reader.read_u16()?;
                let size = reader.read_u16()?;

                let handshake_payload = reader.read_n(size.into())?;

                if encrypted {
                    return Ok(Record {
                        payload: RecordPayloadType::EncryptedHandshake,
                        protocol_version,
                        size,
                        record_type,
                        raw_payload: handshake_payload,
                    });
                }

                let mut reader = Reader::from(handshake_payload, 0);

                let message_type: HandshakeMessageType = reader.read_u8()?.into();
                let _message_size = reader.read_u24()?;

                match message_type {
                    HandshakeMessageType::ServerHello => {
                        let version = reader.read_u16()?;
                        let random = reader.read_n(32)?;
                        let session_id = reader.read_u8_bytes()?;
                        let cipher_suite = reader.read_u16()?.into();
                        let compression_method = reader.read_u8()?;
                        Ok(Record {
                            record_type: RecordType::Handshake,
                            protocol_version,
                            size,
                            raw_payload: handshake_payload,
                            payload: RecordPayloadType::Handshake(HandshakeMessage::ServerHello {
                                version,
                                random,
                                session_id,
                                cipher_suite,
                                compression_method,
                                extensions: {
                                    let mut extensions = vec![];

                                    if reader.has(2) {
                                        let all_extensions = reader.read_u16_bytes()?;
                                        let mut reader = Reader::from(all_extensions, 0);

                                        while reader.has(4) {
                                            let assigned_type = reader.read_u16()?;

                                            extensions.push(match assigned_type {
                                                0x0000 => HandshakeExtensions::ServerName {
                                                    payload: reader.read_u16_bytes()?,
                                                },
                                                0xff01 => HandshakeExtensions::RenegotiationInfo {
                                                    payload: reader.read_u16_bytes()?,
                                                },
                                                0x000b => {
                                                    let mut payload_reader =
                                                        Reader::from(reader.read_u16_bytes()?, 0);
                                                    let mut supported_formats = vec![];
                                                    for _ in 0..payload_reader.read_u8()? {
                                                        supported_formats.push(
                                                            match payload_reader.read_u8()? {
                                                                0x00 => ECPointFormat::Uncompressed,
                                                                0x01 => ECPointFormat::ANSIX962CompressedPrime,
                                                                0x02 => ECPointFormat::ANSIX962CompressedChar2,
                                                                _ => ECPointFormat::Unknown
                                                            },
                                                        );
                                                    }

                                                    HandshakeExtensions::ECPointFormats {
                                                        supported_formats,
                                                    }
                                                }
                                                _ => HandshakeExtensions::Unknown {
                                                    assigned_type,
                                                    payload: reader.read_u16_bytes()?,
                                                },
                                            });
                                        }
                                    }

                                    extensions
                                },
                            }),
                        })
                    }
                    HandshakeMessageType::ServerCertificate => {
                        let mut certificates = vec![];
                        let mut reader = Reader::from(reader.read_u24_bytes()?, 0);

                        while reader.has(3) {
                            certificates.push(reader.read_u24_bytes()?);
                        }

                        Ok(Record {
                            record_type: RecordType::Handshake,
                            protocol_version,
                            size,
                            raw_payload: handshake_payload,
                            payload: RecordPayloadType::Handshake(
                                HandshakeMessage::ServerCertificate { certificates },
                            ),
                        })
                    }
                    HandshakeMessageType::ServerKeyExchange => Ok(Record {
                        record_type: RecordType::Handshake,
                        protocol_version,
                        size,
                        raw_payload: handshake_payload,
                        payload: {
                            let curve_info = match reader.read_u8()? {
                                0x03 => CurveInfo::NamedCurve(reader.read_u16()?),
                                x => CurveInfo::Unknown(x),
                            };

                            let public_key = reader.read_u8_bytes()?;
                            let sig_type = reader.read_u16()?;
                            let sig = reader.read_u16_bytes()?;

                            RecordPayloadType::Handshake(HandshakeMessage::ServerKeyExchange {
                                curve_info,
                                public_key,
                                signature: ServerSignature {
                                    scheme: match sig_type {
                                        0x0601 => ServerSignatureScheme::RSA_PKCS1_SHA512,
                                        x => ServerSignatureScheme::Unknown(x),
                                    },
                                    signature: sig,
                                },
                            })
                        },
                    }),
                    HandshakeMessageType::ServerHelloDone => Ok(Record {
                        record_type: RecordType::Handshake,
                        protocol_version,
                        size,
                        raw_payload: handshake_payload,
                        payload: RecordPayloadType::Handshake(HandshakeMessage::ServerHelloDone {
                            payload: reader.rest(),
                        }),
                    }),
                    HandshakeMessageType::UnknownMessageType(m) => Ok(Record {
                        record_type: RecordType::Handshake,
                        protocol_version,
                        size,
                        raw_payload: handshake_payload,
                        payload: RecordPayloadType::Handshake(HandshakeMessage::UnknownMessage(m)),
                    }),
                }
            }
            RecordType::UnknownRecordType(t) => {
                let protocol_version = reader.read_u16()?;
                let size = reader.read_u16()?;
                let payload = reader.read_n(size.into())?;
                Ok(Record {
                    record_type: RecordType::UnknownRecordType(t),
                    protocol_version,
                    size,
                    raw_payload: payload,
                    payload: RecordPayloadType::UnknownPayload(payload),
                })
            }
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub enum ServerSignatureScheme {
    Unknown(u16),
    RSA_PKCS1_SHA512,
}

#[derive(Clone, Debug)]
pub struct ServerSignature<'a> {
    scheme: ServerSignatureScheme,
    signature: &'a [u8],
}

#[derive(Clone, Debug)]
pub enum CurveInfo {
    NamedCurve(u16),
    Unknown(u8),
}

#[derive(Clone, Debug)]
pub enum HandshakeMessage<'a> {
    UnknownMessage(u8),
    ServerHello {
        version: u16,
        random: &'a [u8],
        session_id: &'a [u8],
        cipher_suite: CiperSuite,
        compression_method: u8, // 0
        extensions: Vec<HandshakeExtensions<'a>>,
    },
    ServerCertificate {
        certificates: Vec<&'a [u8]>,
    },
    ServerKeyExchange {
        curve_info: CurveInfo, // secp256k1 => 0x0017,
        public_key: &'a [u8],
        signature: ServerSignature<'a>,
    },
    ServerHelloDone {
        payload: &'a [u8],
    },
    ServerFinish {
        verify_data: &'a [u8],
    },
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub enum CiperSuite {
    TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
    Unknown(u16),
}

impl From<u16> for CiperSuite {
    fn from(v: u16) -> Self {
        match v {
            x if x == 0xc02f => CiperSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
            x => CiperSuite::Unknown(x),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ECPointFormat {
    Uncompressed,
    ANSIX962CompressedPrime,
    ANSIX962CompressedChar2,
    Unknown,
}

#[derive(Clone, Debug)]
pub enum HandshakeExtensions<'a> {
    Unknown {
        assigned_type: u16,
        payload: &'a [u8],
    },
    ServerName {
        payload: &'a [u8],
    },
    ECPointFormats {
        supported_formats: Vec<ECPointFormat>,
    },
    RenegotiationInfo {
        payload: &'a [u8],
    },
}
