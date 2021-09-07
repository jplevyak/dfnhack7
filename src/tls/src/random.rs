pub struct IcRandom6<'a> {
    random_bytes: &'a [u8],
    index: usize,
}

impl<'a> IcRandom6<'a> {
    pub fn new(random_bytes: &'a [u8]) -> Self {
        Self {
            random_bytes,
            index: 0,
        }
    }
}

impl<'a> rand_core_6::RngCore for IcRandom6<'a> {
    fn next_u32(&mut self) -> u32 {
        let res = ((self.random_bytes[self.index] as u32) << 24)
            | ((self.random_bytes[(self.index + 1) % self.random_bytes.len()] as u32) << 16)
            | ((self.random_bytes[(self.index + 2) % self.random_bytes.len()] as u32) << 8)
            | (self.random_bytes[(self.index + 3) % self.random_bytes.len()] as u32);
        self.index = (self.index + 4) % self.random_bytes.len();
        res
    }
    fn next_u64(&mut self) -> u64 {
        let res = ((self.random_bytes[self.index] as u64) << 56)
            | ((self.random_bytes[(self.index + 1) % self.random_bytes.len()] as u64) << 48)
            | ((self.random_bytes[(self.index + 2) % self.random_bytes.len()] as u64) << 40)
            | ((self.random_bytes[(self.index + 3) % self.random_bytes.len()] as u64) << 32)
            | ((self.random_bytes[(self.index + 4) % self.random_bytes.len()] as u64) << 24)
            | ((self.random_bytes[(self.index + 5) % self.random_bytes.len()] as u64) << 16)
            | ((self.random_bytes[(self.index + 6) % self.random_bytes.len()] as u64) << 8)
            | (self.random_bytes[(self.index + 7) % self.random_bytes.len()] as u64);
        self.index = (self.index + 8) % self.random_bytes.len();
        res
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for i in 0..dest.len() {
            dest[i] = self.random_bytes[self.index];
            self.index = (self.index + 1) % self.random_bytes.len();
        }
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core_6::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}
impl<'a> rand_core_6::CryptoRng for IcRandom6<'a> {}
