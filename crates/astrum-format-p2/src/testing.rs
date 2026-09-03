pub fn xz(bytes: &[u8]) -> Vec<u8> {
    crate::bundle::xz_compress(bytes)
}

pub fn unxz(bytes: &[u8]) -> Vec<u8> {
    crate::bundle::xz_decompress(bytes, u64::MAX).expect("the test file is an xz file")
}
