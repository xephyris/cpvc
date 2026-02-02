#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    DeviceNotFound,
    DeviceAccessFailed(String),
    DeviceEnumerationFailed(String),
    VolumeCaptureFailed(String),
    VolumeSetFailed(String),
    MuteSetFailed(String),
    PlatformUnsupported,
    Placeholder
}