use crate::{device::Device, error::Error};

pub trait API {
    fn get_device_identifiers() -> Result<Vec<(String, String)>, Error> {
        Err(Error::PlatformUnsupported)
    }

    fn get_default_output_dev() -> Result<Device, Error> {
        Err(Error::PlatformUnsupported)
    }

    fn get_sound_devices() -> Result<Vec<String>, Error>;

    fn get_vol() -> Result<f32, Error>;

    fn set_vol(value: f32) -> Result<(), Error>;

    fn get_mute() -> Result<bool, Error>;

    fn set_mute(state: bool) -> Result<(), Error>;

}