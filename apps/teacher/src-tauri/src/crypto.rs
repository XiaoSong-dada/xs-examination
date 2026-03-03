use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn sign_hmac_sha256(secret: &[u8], message: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret)
        .expect("HMAC can take key of any size");
    mac.update(message);
    let result = mac.finalize().into_bytes();
    hex::encode(result)
}
