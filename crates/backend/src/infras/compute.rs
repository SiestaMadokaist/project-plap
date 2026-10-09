#[cfg(feature = "remote-ec2")]
pub mod ec2;
pub mod none;

/// The compute backend the binaries should use: EC2 when `remote-ec2` is on,
/// otherwise a stand-in that offers no regions.
#[cfg(feature = "remote-ec2")]
pub use ec2::EC2MultiRegion as DefaultEngines;
#[cfg(not(feature = "remote-ec2"))]
pub use none::NoEngines as DefaultEngines;
