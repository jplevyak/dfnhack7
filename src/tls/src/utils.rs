pub fn concat(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut c = Vec::from(a);
    c.extend(b);
    c
}
