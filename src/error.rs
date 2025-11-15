#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    DeviceEnumerationFailed(String),
    VolumeCaptureFailed(String),
    Placeholder
}