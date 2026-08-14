use crate::domain::commands::network::NetworkArgs;

/**
 * obtain model from a known remote service (e.g: civitai)
 * store it into remote storage service (e.g: s3)
 */
pub struct Payload {
    args: NetworkArgs,
}
