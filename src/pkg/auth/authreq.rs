pub struct AuthSecret(String);
displayable!(AuthSecret);

pub struct AuthReq {
    pubkey: String,
    signature: String,
    nonce: u32,
}
