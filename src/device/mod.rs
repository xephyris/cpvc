use crate::error::{self, Error};

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

struct Device<T: DeviceTrait> {
    device: T,
}

impl<T> Device<T> 
where 
    T: DeviceTrait
{
    pub fn from_device(device: T) -> Self {
        Device {
            device
        }
    }

    pub fn from_uid(uid: String) -> Result<Self, Error> {
        Ok(Device {
            device: {
                match T::from_uid(uid) {
                    Ok(device) => {
                        device
                    }
                    Err(error ) => {
                        return Err(error)
                    }
                }
            }
        })
    }

    pub fn from_name(name: String) -> Result<Self, Error> {
        Ok(Device {
            device: {
                match T::from_name(name) {
                    Ok(device) => {
                        device
                    }
                    Err(error ) => {
                        return Err(error)
                    }
                }
            }
        })
    }

    pub fn get_vol(&self) -> Result<f32, Error> {
        self.device.get_vol()
    }

    pub fn set_vol(&self, vol: f32) -> Result<(), Error> {
        self.device.set_vol(vol)
    }

    pub fn get_mute(&self) -> Result<bool, Error> {
        self.device.get_mute()
    }

    pub fn set_mute(&self, mute: bool) -> Result<(), Error> {
        self.device.set_mute(mute)
    }

}

#[cfg(test)]
mod test {
    use crate::{coreaudio::device::CoreAudioDevice, device::Device};

    #[test]
    fn test_unified_device() {
        let device = Device::<CoreAudioDevice>::from_name("".to_string()).unwrap();
        dbg!(device.get_mute());
        assert!(false);
    }
}