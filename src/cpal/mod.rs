use cpal::traits::DeviceTrait;
pub use cpal::*;
use crate::{device::Device, error::Error, get_default_output_device, pulseaudio};

pub trait VolumeControlExt {
    fn default_volume_control(&self) -> Result<VolControl, Error>;

    fn device_volume_controls(&self) -> Result<VolControl, Error>;
}

impl VolumeControlExt for cpal::Device {
    fn default_volume_control(&self) -> Result<VolControl, Error> {
        Ok(VolControl::new(crate::get_default_output_device()?))
        // Err(Error::PlatformUnsupported)
    }

    fn device_volume_controls(&self) -> Result<VolControl, Error> {
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
        let device = match device_id.host().to_string().to_lowercase().as_str() {
            "alsa" => {
                #[cfg(target_os = "linux")] {
                    if device_id.id() == "default".to_string() {
                        get_default_output_device()
                    } else {
                        if let Some(card_str) = device_id.id().find("CARD=") && let Some(id_str) = device_id.id().find("DEV=") {
                            // println!("card{card_str} id_str {id_str}" );
                            if let Some(card_num) = device_id.id().chars().nth(card_str + 5).map(|c| c.to_string())
                                && let Some(id_num) = device_id.id().chars().nth(id_str + 4).map(|c| c.to_string()) {
                                match pulseaudio::convert_alsa_id(card_num, id_num) {
                                    Ok(dev_id) => {
                                        Device::from_uid(dev_id)
                                    },
                                    Err(_) => {
                                        Err(Error::DeviceNotFound)
                                    }
                                }
                            } else {
                                Err(Error::DeviceNotFound)
                            }
                        } else {
                            Err(Error::DeviceNotFound)
                        }
                    }
                } 
                #[cfg(not(target_os="linux"))]
                Err(Error::PlatformUnsupported)              
            },
            "coreaudio" => {
                Device::from_uid(device_id.id().to_string())
            },
            "wasapi" => {
                Device::from_uid(device_id.id().to_string())
            },
            _ => {
                Err(Error::PlatformUnsupported)
            }
        }?;

        Ok(VolControl {
            device
        })
    }

    pub fn consume(self) -> Device {
        self.device
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

use cpal::{DeviceId, traits::{DeviceTrait, HostTrait}};

    use crate::cpal::VolumeControlExt;

    // use crate::cpal::{VolumeControlExt, uid_to_hw_id};

    #[test]
    fn cpal_get_device_name() {
        use cpal::traits::HostTrait;
        use crate::cpal::traits::DeviceTrait;
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available");
        // let id = &DeviceId::from_str("").unwrap();
        // let device = host.device_by_id(id).expect("no output device available");
        
        println!("{}", device.description().unwrap().name());
        println!("{:?}", host.id());
        println!("DEVICE UID: {:?}, DEVICE CONVERTED HW_ID", device.id());
        assert!(false);
    }
    
    #[test]
    fn cpal_test_device_general() {
        use cpal::traits::HostTrait;
        use crate::cpal::traits::DeviceTrait;
        let host = cpal::default_host();
        // let device = host.default_output_device().expect("no output device available");
        let id = &DeviceId::from_str("alsa:hw:CARD=0,DEV=0").unwrap();
        let device = host.device_by_id(id).expect("no output device available");
        println!("{}", device.description().unwrap().name());
        let vol_control =  device.device_volume_controls().unwrap();
        let raw_device = vol_control.consume();
        dbg!(raw_device.get_name());
        dbg!(raw_device.set_mute(false));
        dbg!(raw_device.get_vol());
        dbg!(raw_device.set_vol(0.1));
        assert!(false);
    }

    #[test]
    fn cpal_device_vol() {
        use cpal::traits::HostTrait;
        use crate::cpal::traits::DeviceTrait;
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available");
        // let id = &DeviceId::from_str("").unwrap();
        // let device = host.device_by_id(id).expect("no output device available");
        println!("{}", device.description().unwrap().name());
        let vol_control =  device.device_volume_controls().unwrap();
        dbg!(vol_control.set_vol(0.10));
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

    #[test]
    fn cpal_current_id() {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available");
        println!("{}", device.description().unwrap().name());
        println!("{:?}", device.id());
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