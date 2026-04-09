pub use cpal::*;
use crate::{debug_eprintln, device::Device, error::Error};

pub trait VolumeControlExt {
    fn default_volume_control(&self) -> Result<VolControl, Error>;
}

impl VolumeControlExt for cpal::Device {
    fn default_volume_control(&self) -> Result<VolControl, Error> {
        VolControl::new(crate::get_default_output_device()?);
        Err(Error::PlatformUnsupported)
    }
}

pub struct VolControl {
    device: Device,
}

impl VolControl {
    pub fn new(device: Device) -> VolControl {
        VolControl {
            device
        }
    }

    pub fn from_cpal_id(device_id: DeviceId) -> Result<Self, Error> {
        let device = match device_id.0.to_string().as_str() {
            // "alsa" => {
            //     Device::from_uid(device_id.1)
            // },
            "coreaudio" => {
                Device::from_uid(device_id.1)
            },
            "wasapi" => {
                Device::from_uid(device_id.1)
            },
            _ => {
                Err(Error::PlatformUnsupported)
            }
        }?;

        Ok(VolControl {
            device
        })
    }

    pub fn set_vol(&self, val: f32) -> Result<(), Error> {
        self.device.set_vol(val)
    }
    pub fn get_vol(&self) -> Result<f32, Error>  {
        self.device.get_vol()
    }
    pub fn set_mute(&self, mute: bool) -> Result<(), Error> {
        self.device.set_mute(mute)
    }

    pub fn is_mute(&self) -> Result<bool, Error> {
        self.device.get_mute()
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use cpal::traits::{DeviceTrait, HostTrait};

    use crate::cpal::VolumeControlExt;

    // use crate::cpal::{VolumeControlExt, uid_to_hw_id};

    #[test]
    fn cpal_get_device_name() {
        use cpal::traits::HostTrait;
        use crate::cpal::traits::DeviceTrait;
        let host = cpal::default_host();
        println!("{:?}", host.id());
        let device = host.default_output_device().expect("no output device available");
        println!("{}", device.name().unwrap());
        // println!("DEVICE UID: {:?}, DEVICE CONVERTED HW_ID: {:?}", device.id(), uid_to_hw_id(device.id().unwrap().1));
        assert!(false);
    }

    #[test]
    fn cpal_device_vol() {
        use cpal::traits::HostTrait;
        use crate::cpal::traits::DeviceTrait;
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available");
        println!("{}", device.name().unwrap());
        let vol_control =  device.default_volume_control().unwrap();
        dbg!(vol_control.set_vol(0.20));
        dbg!(vol_control.get_vol());
        // println!("{:?}", device.default_volume_control().unwrap());
        assert!(false);
    }

    #[test]
    fn cpal_devices() {
        let host = cpal::default_host();
        for device in host.devices().unwrap() {
            println!("{}", device.name().unwrap_or_default());
            println!("{:?}", device.id().unwrap());
        }
        assert!(false);
    }

    // #[test]
    // fn cpal_from_uid() {
    //     use cpal::traits::`{HostTrait, DeviceTrait};
    //     let host = cpal::default_host();
    //     let device = host.default_output_device().unwrap();
    //     let vol_control =  device.default_volume_control().unwrap();
    //     dbg!(vol_control.set_vol(0.20));
    //     println!("{:?}", device.default_volume_control().unwrap());
    //     assert!(false);
    // }
    
}