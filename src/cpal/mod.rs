use cpal::traits::DeviceTrait;
pub use cpal::*;
use crate::{debug_eprintln, device::Device, error::Error};

pub trait VolumeControlExt {
    fn default_volume_control(&self) -> Result<VolControl, Error>;

    fn device_voltume_controls(&self) -> Result<VolControl, Error>;
}

impl VolumeControlExt for cpal::Device {
    fn default_volume_control(&self) -> Result<VolControl, Error> {
        Ok(VolControl::new(crate::get_default_output_device()?))
        // Err(Error::PlatformUnsupported)
    }

    fn device_voltume_controls(&self) -> Result<VolControl, Error> {
        VolControl::from_cpal_id(self.id().map_err(|e| Error::External(e.to_string()))?)
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
        let device = match device_id.0.to_string().to_lowercase().as_str() {
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
        println!("{}", device.description().unwrap().name());
        println!("DEVICE UID: {:?}, DEVICE CONVERTED HW_ID", device.id());
        assert!(false);
    }

    #[test]
    fn cpal_device_vol() {
        use cpal::traits::HostTrait;
        use crate::cpal::traits::DeviceTrait;
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available");
        println!("{}", device.description().unwrap().name());
        let vol_control =  device.default_volume_control().unwrap();
        dbg!(vol_control.set_vol(0.20));
        dbg!(vol_control.get_vol());
        // println!("{:?}", device.default_volume_control().unwrap());
        assert!(false);
    }

    #[test]
    fn cpal_device_mute() {
        use cpal::traits::HostTrait;
        use crate::cpal::traits::DeviceTrait;
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available");
        println!("{}", device.description().unwrap().name());
        let vol_control =  device.default_volume_control().unwrap();
        dbg!(vol_control.set_mute(true));
        dbg!(vol_control.is_mute());
        assert!(false);
    }

    #[test]
    fn cpal_devices() {
        let host = cpal::default_host();
        for device in host.devices().unwrap() {
            println!("{}", device.description().unwrap().name());
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