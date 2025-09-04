pub struct AudioDevice {
    pub device_name: String,
    pub hw_name: String,
    pub device_id: u32,
    channels: u32,
}

impl AudioDevice {
    pub fn get_default_device() -> Result<AudioDevice, Error> {
       Err(Error::UnsupportedOS)
    }
}

pub enum Error {
    UnsupportedOS
}