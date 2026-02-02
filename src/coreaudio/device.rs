// #[cfg(not(target_os="macos"))] // Should be disabled, for testing
#[cfg(target_os="macos")]
mod device {
    use crate::{debug_eprintln, error::Error};
    #[cfg(target_os="macos")]
    use {
        std::ffi::c_void,
        std::ptr::{null},
        std::mem::{size_of},
        std::ptr::NonNull,
        objc2_core_audio::{
            AudioObjectGetPropertyData, AudioObjectSetPropertyData, AudioObjectGetPropertyDataSize,
            AudioObjectPropertyAddress,
            kAudioObjectPropertyElementMain,
            kAudioDevicePropertyScopeOutput, kAudioDevicePropertyMute,
            kAudioDevicePropertyVolumeScalar,
        },
        crate::coreaudio
    };

    pub struct CoreAudioDevice {
        device_id: u32,
    }

    impl CoreAudioDevice {
        pub fn from_name(name: String) -> Result<Self, Error> {
            if let Ok(identifiers) = coreaudio::get_device_identifiers() {
                for (id, dev_name) in identifiers {
                    if dev_name == name {
                        return Ok(CoreAudioDevice { device_id: id })
                    }
                }
                return Err(Error::Placeholder);
            }
            Err(Error::DeviceEnumerationFailed("Failed to identify devices".to_string()))
        }

        pub fn from_uid(uid: String) -> Result<Self, Error> {
            let hw_id = coreaudio::uid_to_hw_id(uid)?;
            Ok(CoreAudioDevice {
                device_id: hw_id,
            })
        }
        
        pub fn from_hw_id(hw_id: u32) -> Result<Self, Error> {
            if let Ok(identifiers) = coreaudio::get_device_identifiers() {
                for (id, _dev_name) in identifiers {
                    if hw_id == id {
                        return Ok(CoreAudioDevice { device_id: id });
                    }
                }
                return Err(Error::Placeholder);
            }
            Err(Error::DeviceEnumerationFailed("Failed to identify devices".to_string()))
        }

        pub fn get_device_hw_id(&self) -> u32 {
            self.device_id
        }

        pub fn get_name(&self) -> Result<String, Error> {
            coreaudio::get_device_name(self.device_id)
        }

        pub fn get_hardware_device_name(&self) -> Result<String, Error> {
            coreaudio::get_hw_name(self.device_id)
        }

        pub fn get_vol(&self) -> Result<f32, Error> {
            let mut vol= 0;
            let device_id = self.device_id;

            // Check if Muted
            if self.get_mute()? == false {
                let device_details = coreaudio::get_output_device_details(device_id);
                if device_details.is_ok() {
                    let channel_count = device_details.unwrap().mChannelsPerFrame;
                    let mut total_volume: f32 = 0.0;
                    let mut total_channels = 0;
                    let mut channel_volume: f32 = 0.0;
                    let mut volume_data_size = size_of::<f32>() as u32;

                    for channel in 0..=channel_count {
                        let mut volume_property_address_channel = AudioObjectPropertyAddress {
                            mSelector: kAudioDevicePropertyVolumeScalar,
                            mScope: kAudioDevicePropertyScopeOutput,
                            mElement: channel,
                        };

                        unsafe {
                            let get_volume_data_size_status = AudioObjectGetPropertyDataSize(
                                    device_id,
                                    NonNull::new_unchecked(&mut volume_property_address_channel as *mut _),
                                    0,
                                    null(),
                                    NonNull::new_unchecked(&mut volume_data_size as *mut _),
                                );
                            if get_volume_data_size_status == 0 {
                                let get_volume_status = AudioObjectGetPropertyData(
                                    device_id,
                                    NonNull::new_unchecked(&mut volume_property_address_channel as *mut _),
                                    0,
                                    null(),
                                    NonNull::new_unchecked(&mut volume_data_size as *mut _),
                                    NonNull::new_unchecked(&mut channel_volume as *mut _ as *mut c_void));
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
                        vol = (total_volume as u32 / total_channels) as u8;
                    } else {
                        return Err(Error::VolumeCaptureFailed(format!("Failed to capture volume information from any of the channels in the device.\nChannels detected: {}\nChannels captured{}", channel_count, total_channels)));
                    }
                }
            } else {
                vol = 0;
            }
            return Ok(vol as f32 / 100.0)
        }

        pub fn set_vol(&self, value: f32) -> Result<(), Error> {
            let mut status = None;
            let device_id = self.device_id;
            let device_details = coreaudio::get_output_device_details(device_id);

            if device_details.is_ok() {
                let channel_count = device_details.unwrap().mChannelsPerFrame;

                let mut volume = value;
                let volume_data_size = size_of::<f32>() as u32;

                for channel in 0..=channel_count {
                    debug_eprintln(&format!("channel {}", channel));
                    let mut volume_property_address_channel = AudioObjectPropertyAddress {
                        mSelector: kAudioDevicePropertyVolumeScalar,
                        mScope: kAudioDevicePropertyScopeOutput,
                        mElement: channel,
                    };

                    unsafe {
                        let change_volume_status = AudioObjectSetPropertyData(device_id,
                            NonNull::new_unchecked(&mut volume_property_address_channel as *mut _),
                            0, null(),
                            volume_data_size, NonNull::new_unchecked(&mut volume as *mut _ as *mut _));
                        if change_volume_status != 0 {
                            debug_eprintln(&format!("Failed to change volume on channel {} (This may be normal behavior)", if channel == 0 {"0 (Master Channel)".to_string()} else {channel.to_string()}));
                        }
                    }
                }

                // Mute then unmute hardware device so software sound level will sync with hardware sound level
                self.set_mute(true)?;
                self.set_mute(false)?;

                if value == 0.0 {
                    self.set_mute(true)?;
                }

                status.replace(true);

            } else {
                status.replace(false);
            }
        if status.unwrap_or(false) {Ok(())} else { Err(Error::Placeholder) }
        }

        pub fn get_mute(&self) -> Result<bool, Error> {
            let mut mute = 0;
            let device_id = self.device_id;
            let mut mute_property_address = AudioObjectPropertyAddress {
                        mSelector: kAudioDevicePropertyMute,
                        mScope: kAudioDevicePropertyScopeOutput,
                        mElement: kAudioObjectPropertyElementMain
                    };
            let mut mute_data_size = size_of::<u32>() as u32;
            unsafe {
                let mute_status = AudioObjectGetPropertyData(device_id,
                    NonNull::new_unchecked(&mut mute_property_address as *mut _),
                    0, null(),
                    NonNull::new_unchecked(&mut mute_data_size as *mut _), NonNull::new_unchecked(&mut mute as *mut _ as *mut _));
                if mute_status != 0{
                    debug_eprintln("failed to gather mute status");
                }
            }
            if mute == 1 { Ok(true) } else { Ok(false) }
        }

        pub fn set_mute(&self, state: bool) -> Result<(), Error> {
            let mut status = true;
            let device_id = self.device_id;
            let mut mute_property_address = AudioObjectPropertyAddress {
                mSelector: kAudioDevicePropertyMute,
                mScope: kAudioDevicePropertyScopeOutput,
                mElement: kAudioObjectPropertyElementMain
            };
            let mute_data_size = size_of::<u32>() as u32;
            let mut mute = match state {
                true => {
                    1
                },
                false => {
                    0
                }
            };
            unsafe {
                let mute_status = AudioObjectSetPropertyData(device_id,
                    NonNull::new_unchecked(&mut mute_property_address as  *mut _),
                    0, null(),
                    mute_data_size, NonNull::new_unchecked(&mut mute as *mut _ as *mut _));
                if mute_status != 0 {
                    status = false;
                }
            }
            if status {
                Ok(())
            } else {
                Err(Error::Placeholder)
            }
        }

    }
}

// Stubs for cross platform compile
#[cfg(not(target_os="macos"))]
// #[cfg(target_os="macos")] // Should be disabled, for testing
mod device {
    use crate::{debug_eprintln, error::Error};
    #[derive(Default)]
    pub struct CoreAudioDevice {
        device_id: u32,
    }

    impl CoreAudioDevice {
        pub fn from_name(name: String) -> Result<Self, Error> {
            Err(Error::PlatformUnsupported)
        }

        pub fn from_uid(uid: String) -> Result<Self, Error> {
            Err(Error::PlatformUnsupported)
        }
        
        pub fn from_hw_id(hw_id: u32) -> Result<Self, Error> {
            Err(Error::PlatformUnsupported)
        }

        pub fn get_device_hw_id(&self) -> u32 {
            0
        }

        pub fn get_name(&self) -> Result<String, Error> {
            Err(Error::PlatformUnsupported)
        }

        pub fn get_hardware_device_name(&self) -> Result<String, Error> {
            Err(Error::PlatformUnsupported)
        }

        pub fn get_vol(&self) -> Result<f32, Error> {
            Err(Error::PlatformUnsupported)
        }

        pub fn set_vol(&self, value: f32) -> Result<(), Error> {
            Err(Error::PlatformUnsupported)
        }

        pub fn get_mute(&self) -> Result<bool, Error> {
            Err(Error::PlatformUnsupported)
        }

        pub fn set_mute(&self, state: bool) -> Result<(), Error> {
            Err(Error::PlatformUnsupported)
        }
    }
}

pub(crate) use device::*; 