use sha2::Digest;
use std::iter::FromIterator;

pub struct Transcript {
    messages: Vec<u8>,
    message_count: usize,
}

impl Transcript {
    pub fn new() -> Self {
        Transcript {
            messages: vec![],
            message_count: 0,
        }
    }

    pub fn add(&mut self, message: &[u8]) {
        self.messages.extend_from_slice(message);
        self.message_count += 1;
    }

    pub fn get_hash(&self) -> Vec<u8> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&self.messages);

        Vec::from_iter(hasher.finalize().into_iter())
    }
}
