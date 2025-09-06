
#[derive(Debug)]
pub struct AudioDevice {
    pub device_name: String,
    pub hw_name: String,
    pub device_id: u32,
    channels: u32,
}

impl AudioDevice {
    pub fn get_default_device() -> Result<AudioDevice, Error> {

        #[cfg(target_os="macos")] {
            use crate::{capture_output_device_id, get_device_name, get_output_device_details};

            let captured_device_id = capture_output_device_id();
            if let Ok(device_id) = captured_device_id {
                let mut device_name = String::new();
                let channels;
                let name = get_device_name(captured_device_id.unwrap());
                if name.is_ok() {
                    device_name.push_str(&name.unwrap());
                }
                
                let device_stats = get_output_device_details(device_id);
                if let Ok(stats) = device_stats {
                    channels = stats.mChannelsPerFrame;
                } else {
                    channels = 0;
                }
                return Ok(AudioDevice {
                    device_name: device_name.clone(),
                    device_id,
                    hw_name: device_name,
                    channels
                });
            }
        }

        Err(Error::UnsupportedOS)
    }

    pub fn get_device_from_id(device_id: u32) -> Result<AudioDevice, Error> {
        #[cfg(target_os="macos")] {
            use crate::{get_device_name, get_output_device_details};

            let mut device_name = String::new();
            let channels;
            let name = get_device_name(device_id);
            if name.is_ok() {
                device_name.push_str(&name.unwrap());
            }
            
            let device_stats = get_output_device_details(device_id);
            if let Ok(stats) = device_stats {
                channels = stats.mChannelsPerFrame;
            } else {
                channels = 0;
            }
            return Ok(AudioDevice {
                device_name: device_name.clone(),
                device_id,
                hw_name: device_name,
                channels
            });

            Err(Error::UnsupportedOS)
        }
    }

    pub fn get_device_from_name(name: String) -> Result<AudioDevice, Error> {
        #[cfg(target_os="macos")] {
            use crate::{get_device_name, get_output_device_details, scan::scan_devices};

            if let Some(device_id) = scan_devices().remove(&name) {
                let mut device_name = String::new();
                let channels;
                let name = get_device_name(device_id);
                if name.is_ok() {
                    device_name.push_str(&name.unwrap());
                }
                
                let device_stats = get_output_device_details(device_id);
                if let Ok(stats) = device_stats {
                    channels = stats.mChannelsPerFrame;
                } else {
                    channels = 0;
                }
                return Ok(AudioDevice {
                    device_name: device_name.clone(),
                    device_id,
                    hw_name: device_name,
                    channels
                });
            }
            Err(Error::UnsupportedOS)
        }
    }


}

#[derive(Debug)]
pub enum Error {
    UnsupportedOS
}

#[cfg(test)]
mod tests {
    use crate::device::AudioDevice;

    #[test]
    fn get_audio_device() {
        dbg!(AudioDevice::get_default_device());
        dbg!(AudioDevice::get_device_from_name("External Headphones".to_string()));
        assert!(false);
    }
}