use ring::digest;

pub fn do_sha256(data: Vec<u8>) -> Vec<u8> {
    digest::digest(&digest::SHA256, data.as_slice())
        .as_ref()
        .to_vec()
}