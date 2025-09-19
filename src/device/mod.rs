#[cfg(target_os="macos")]
use objc2_core_audio::{
        AudioObjectGetPropertyData, AudioObjectSetPropertyData, AudioObjectGetPropertyDataSize,
        AudioObjectID, AudioObjectPropertyAddress,
        kAudioHardwarePropertyDefaultOutputDevice, kAudioObjectSystemObject,
        kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyElementMain,
        kAudioDevicePropertyScopeOutput, kAudioDevicePropertyMute,
        kAudioDevicePropertyVolumeScalar, kAudioDevicePropertyDeviceNameCFString,
        kAudioDevicePropertyStreamFormat, kAudioObjectPropertyScopeOutput,
        kAudioHardwarePropertyDevices, kAudioDevicePropertyStreams,
        kAudioObjectPropertyScopeInput,
};
#[cfg(target_os="windows")]
use windows::Win32::Media::Audio::IMMDevice;
use crate::debug_eprintln;

#[derive(Debug, Clone)]
enum DeviceId {
    MacOS(u32),
    Windows(String),
}

#[derive(Debug)]
pub struct AudioDevice {
    pub device_name: String,
    pub hw_name: String,
    pub device_id: DeviceId,
    channels: u32,
    vol_ctl: VolControl,
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
                    device_id: DeviceId::MacOS(device_id),
                    hw_name: device_name,
                    channels,
                    vol_ctl: VolControl::new(DeviceId::MacOS(device_id), channels)
                });
            } else {
                return Err(Error::DeviceCaptureFailed);
            }
        }

        Err(Error::UnsupportedOS)
    }

    pub fn get_device_from_id(device_id: DeviceId) -> Result<AudioDevice, Error> {
        #[cfg(target_os="macos")] {
            use crate::{get_device_name, get_output_device_details};
            let device_id = match device_id {
                DeviceId::MacOS(id_num) => {
                    id_num
                }
                _ => {
                    return Err(Error::DeviceCaptureFailed);
                }
            };
            let mut device_name = String::new();
            let channels;
            let name = get_device_name(device_id);
            if name.is_ok() {
                device_name.push_str(&name.unwrap());
            } else {
                return Err(Error::DeviceNotFound);
            }
            
            let device_stats = get_output_device_details(device_id);
            if let Ok(stats) = device_stats {
                channels = stats.mChannelsPerFrame;
            } else {
                return Err(Error::DeviceDetailsCaptureFailed);
            }
            return Ok(AudioDevice {
                device_name: device_name.clone(),
                device_id: DeviceId::MacOS(device_id),
                hw_name: device_name,
                channels,
                vol_ctl: VolControl::new(device_id, channels)
            });

        }
        #[cfg(target_os="windows")] {
            let device_id = match device_id {
                DeviceId::Windows(pwstr) => {
                    pwstr
                },
                _ => {
                    return Err(Error::DeviceCaptureFailed);
                }
            };
            let name;
            let hw_name;
            let channels;


            unsafe {
                use windows::{core::PWSTR, Win32::{Media::Audio::Endpoints::IAudioEndpointVolume, System::Com::{CLSCTX_ALL, STGM_READ}}};

                use crate::get_enumerator;

                let mut id = format!("{}\0", device_id).encode_utf16().collect::<Vec<u16>>();
                let pwstr = PWSTR(id.as_mut_ptr());

                let enumerator = get_enumerator();
                let device = match enumerator.GetDevice(pwstr) {
                    Ok(device) => {
                        device
                    },
                    Err(_) => {
                        return Err(Error::DeviceCaptureFailed);
                    },
                };
                let volume_controls = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).unwrap();
                channels = volume_controls.GetChannelCount().unwrap();
                let result = device.OpenPropertyStore(STGM_READ);
                match result {
                    Ok(properties) => {
                        use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;

                        name = properties.GetValue(&PKEY_Device_FriendlyName).unwrap().to_string();
                        hw_name = name.clone()
                        // dbg!(properties.GetValue(&PKEY_Device_FriendlyName));
                    },
                    Err(error) => {
                        panic!("{}", error);
                    }
                }
            }
            
            return Ok(AudioDevice{
                device_name: name,
                hw_name,
                device_id: DeviceId::Windows(device_id.clone()),
                channels,
                vol_ctl: VolControl::new(DeviceId::Windows(device_id.clone()), channels),
            })

        }
        Err(Error::UnsupportedOS)
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
                } else {
                    return Err(Error::DeviceDetailsCaptureFailed);
                }
                
                let device_stats = get_output_device_details(device_id);
                if let Ok(stats) = device_stats {
                    channels = stats.mChannelsPerFrame;
                } else {
                    return Err(Error::DeviceDetailsCaptureFailed);
                }
                return Ok(AudioDevice {
                    device_name: device_name.clone(),
                    device_id: DeviceId::MacOS(device_id),
                    hw_name: device_name,
                    channels,
                    vol_ctl: VolControl::new(DeviceId::MacOS(device_id), channels)
                });
            }
            return Err(Error::DeviceNotFound);        
        }
        Err(Error::UnsupportedOS)
    }

    pub fn default_volume_control(&self) -> VolControl {
        self.vol_ctl.clone()
    }

}

#[derive(Debug, Clone)]
pub struct VolControl {
    hw_id: DeviceId,
    channels: u32,

}

impl VolControl {
    fn new(hw_id: DeviceId, channels: u32) -> VolControl {
        VolControl { hw_id, channels }
    }

    pub fn set_vol(&self, val: f32) -> bool {
        let mut success = Some(false);
        #[cfg(target_os="macos")]{
            use std::ptr::{null, NonNull};
            let channel_count = self.channels;

            let volume_data_size = size_of::<f32>() as u32;

            for channel in 0..=channel_count {
                debug_eprintln(&format!("channel {}", channel));
                let volume_property_address_channel = AudioObjectPropertyAddress {
                    mSelector: kAudioDevicePropertyVolumeScalar,
                    mScope: kAudioDevicePropertyScopeOutput,
                    mElement: channel,
                };

                unsafe {
                    let change_volume_status = AudioObjectSetPropertyData(self.hw_id,
                        NonNull::new_unchecked(&volume_property_address_channel as *const _ as *mut _),
                        0, null(),
                        volume_data_size, NonNull::new_unchecked( &val as *const _ as *mut _));
                    if change_volume_status != 0 {
                        debug_eprintln(&format!("Failed to change volume on channel {} (This may be normal behavior)", if channel == 0 {"0 (Master Channel)".to_string()} else {channel.to_string()}));
                    }
                }
            }

            let mute_property_address = AudioObjectPropertyAddress {
                mSelector: kAudioDevicePropertyMute,
                mScope: kAudioDevicePropertyScopeOutput,
                mElement: kAudioObjectPropertyElementMain
            };

            let mut sync_status = true;
            // Mute then unmute hardware device so software sound level will sync with hardware sound level
            for mute in (0..=1 as u32).rev() {
                let mute_data_size = size_of::<u32>() as u32;
                unsafe {
                    let mute_status = AudioObjectSetPropertyData(self.hw_id,
                        NonNull::new_unchecked(&mute_property_address as *const _ as *mut _),
                        0, null(),
                        mute_data_size, NonNull::new_unchecked(&mute as *const _ as *mut _));
                    if mute_status != 0 {
                        sync_status = false;
                    }
                }
            }
            if success.is_none() {
                success.replace(sync_status);
            }
        }
        success.unwrap_or(false)
    }
    pub fn get_vol(&self) -> f32 {
        let mut vol = 0.0;
        #[cfg(target_os="macos")] {
            use std::ptr::{NonNull, null};
            let mute_property_address = AudioObjectPropertyAddress {
                    mSelector: kAudioDevicePropertyMute,
                    mScope: kAudioDevicePropertyScopeOutput,
                    mElement: kAudioObjectPropertyElementMain
                };

            // Check if Muted
            let channel_count = self.channels;
            let mut total_volume: f32 = 0.0;
            let mut total_channels = 0;
            let mut channel_volume: f32 = 0.0;
            let mut volume_data_size = size_of::<f32>() as u32;

            for channel in 0..=channel_count {
                let volume_property_address_channel = AudioObjectPropertyAddress {
                    mSelector: kAudioDevicePropertyVolumeScalar,
                    mScope: kAudioDevicePropertyScopeOutput,
                    mElement: channel,
                };

                unsafe {


                    let get_volume_data_size_status = AudioObjectGetPropertyDataSize(
                            self.hw_id,
                            NonNull::new_unchecked(&volume_property_address_channel as *const _ as *mut _),
                            0,
                            null(),
                            NonNull::new_unchecked(&mut volume_data_size as *const _ as *mut _),
                        );
                    if get_volume_data_size_status == 0 {
                        let get_volume_status = AudioObjectGetPropertyData(
                            self.hw_id,
                            NonNull::new_unchecked(&volume_property_address_channel as *const _ as *mut _),
                            0,
                            null(),
                            NonNull::new_unchecked(&volume_data_size as *const _ as *mut _),
                            NonNull::new_unchecked(&mut channel_volume as *mut _ as *mut _));

                        if get_volume_status != 0 {
                            debug_eprintln(&format!("Failed to get volume on channel {} (This may be normal behavior)", if channel == 0 {"0 (Master Channel)".to_string()} else {channel.to_string()}));
                        } else {
                            total_channels += 1;
                            total_volume += channel_volume;
                        }
                    } else {
                        debug_eprintln(&format!("Failed to get volume data size on channel {} (This may be normal behavior)", if channel == 0 {"0 (Master Channel)".to_string()} else {channel.to_string()}));
                    }
                }
            }
            if total_channels > 0 {
                total_volume *= 100.0;
                total_volume = total_volume.round();
                vol = (total_volume as u32 / total_channels) as f32;
            }
        }
        vol.into()
    }
    pub fn set_mute(&self, mute: bool) -> bool {
        let mut status = false;
        #[cfg(target_os="macos")] {
            let mute_property_address = AudioObjectPropertyAddress {
                        mSelector: kAudioDevicePropertyMute,
                        mScope: kAudioDevicePropertyScopeOutput,
                        mElement: kAudioObjectPropertyElementMain
                    };
            let mute_data_size = size_of::<u32>() as u32;
            let mute = match mute {
                true => {
                    1
                },
                false => {
                    0
                }
            };
            unsafe {
                use std::ptr::{NonNull, null};
                let mute_status = AudioObjectSetPropertyData(self.hw_id,
                    NonNull::new_unchecked(&mute_property_address as *const _ as *mut _),
                    0, null(),
                    mute_data_size, NonNull::new_unchecked(&mute as *const _ as *mut _));
                if mute_status != 0 {
                    status = false;
                }
            }
        }
        status
    }

    pub fn is_mute(&self) -> Result<bool, VolumeError> {
        let mut mute:u32 = 0;
        #[cfg(target_os="macos")] {
            let mute_property_address = AudioObjectPropertyAddress {
                        mSelector: kAudioDevicePropertyMute,
                        mScope: kAudioDevicePropertyScopeOutput,
                        mElement: kAudioObjectPropertyElementMain
                    };
            let mut mute_data_size = size_of::<u32>() as u32;

            unsafe {
                use std::ptr::{NonNull, null};
                let mute_status = AudioObjectGetPropertyData(self.hw_id,
                    NonNull::new_unchecked(&mute_property_address as *const _ as *mut _),
                    0, null(),
                    NonNull::new_unchecked(&mut mute_data_size as *mut _), 
                    NonNull::new_unchecked(&mut mute as *mut _ as *mut _));
                if mute_status != 0 {
                    return Err(VolumeError::MuteStatusCaptureFailed);
                }
            }
        }
        Ok(match mute {
            1 => {
                true
            }
            _ => {
                false
            }
        })
    }

    #[cfg(target_os="windows")] 
    fn get_hw_device(&self) -> Result<IMMDevice, Error> {
        let device_id = match self.hw_id.clone() {
            DeviceId::Windows(pwstr) => {
                pwstr
            },
            _ => {
                return Err(Error::DeviceCaptureFailed);
            }
        };

        unsafe {
            use windows::core::PWSTR;

            use crate::get_enumerator;

            let mut id = format!("{}\0", device_id).encode_utf16().collect::<Vec<u16>>();
            let pwstr = PWSTR(id.as_mut_ptr());

            let enumerator = get_enumerator();
            match enumerator.GetDevice(pwstr) {
                Ok(device) => {
                    Ok(device)
                },
                Err(_) => {
                    Err(Error::DeviceCaptureFailed)
                },
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    UnsupportedOS,
    DeviceCaptureFailed,
    DeviceDetailsCaptureFailed,
    DeviceNotFound,
}

#[derive(Debug)]
pub enum VolumeError {
    MuteStatusCaptureFailed,
}

#[cfg(test)]
mod tests {
    use crate::device::AudioDevice;

    #[test]
    fn get_audio_device() {
        dbg!(AudioDevice::get_default_device());
        dbg!(AudioDevice::get_device_from_name("External Headphones".to_string()));
        #[cfg(target_os="windows")] {
            use crate::device::DeviceId;
            
        }
        assert!(false);
    }

    #[test]
    fn get_audio_device_volume() {
        let device = AudioDevice::get_device_from_name("Mac mini Speakers".to_string()).unwrap();
        let mut vol_ctl = device.default_volume_control();
        dbg!(vol_ctl.get_vol());
        dbg!(device);
        assert!(false);
    }

    #[test]
    fn check_muted() {
        // let device = AudioDevice::get_device_from_name("Mac mini Speakers".to_string()).unwrap();
        let device = AudioDevice::get_default_device().unwrap();
        let mut vol_ctl = device.default_volume_control();
        dbg!(vol_ctl.is_mute());
        dbg!(vol_ctl.set_mute(true));
        dbg!(device);
        assert!(false);
    }
}
