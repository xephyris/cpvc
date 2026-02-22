use crate::error::Error;

pub trait DeviceTrait {
    fn from_name(name: String) -> Result<Self, Error> where Self: Sized {
        Err(Error::PlatformUnsupported)
    }

    fn from_uid(uid: String) -> Result<Self, Error> where Self: Sized {
        Err(Error::PlatformUnsupported)
    }

    fn get_name(&self) -> Result<String, Error> {
        Err(Error::PlatformUnsupported)
    }

    fn get_vol(&self) -> Result<f32, Error> {
        Err(Error::PlatformUnsupported)
    }

    fn set_vol(&self, value: f32) -> Result<(), Error> {
        Err(Error::PlatformUnsupported)
    }

    fn get_mute(&self) -> Result<bool, Error> {
        Err(Error::PlatformUnsupported)
    }

    fn set_mute(&self, state: bool) -> Result<(), Error> {
        Err(Error::PlatformUnsupported)
    }

}

struct Device {

}