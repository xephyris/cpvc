use objc2_core_audio::kAudioHardwarePropertyDeviceForUID;

use crate::{DeviceType, VolumeControl, VolumeError, debug_eprintln, error::Error};

#[cfg(target_os="macos")]
use {
    std::ffi::c_void,
    std::ptr::{null, null_mut},
    std::mem::{size_of},
    std::ptr::NonNull,
    core_foundation::{base::TCFType, string::{CFString, CFStringRef}},
    objc2_core_audio_types::{AudioStreamBasicDescription},
    objc2_core_audio::{
        AudioObjectGetPropertyData, AudioObjectSetPropertyData, AudioObjectGetPropertyDataSize,
        AudioObjectID, AudioObjectPropertyAddress,
        kAudioHardwarePropertyDefaultOutputDevice, kAudioObjectSystemObject,
        kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyElementMain,
        kAudioDevicePropertyScopeOutput, kAudioDevicePropertyMute,
        kAudioDevicePropertyVolumeScalar, kAudioDevicePropertyDeviceNameCFString,
        kAudioDevicePropertyStreamFormat, kAudioObjectPropertyScopeOutput,
        kAudioHardwarePropertyDevices, kAudioDevicePropertyStreams,
        kAudioObjectPropertyScopeInput,
    },
};

// TODO Create CoreAudioExpanded struct for non essential features

pub struct CoreAudio {
}

impl VolumeControl for CoreAudio {
    fn get_sound_devices() -> Result<Vec<String>, Error> {
        let mut devices:Vec<String> = Vec::new();
        let audio_devices_count_address =  AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDevices,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        };

        let mut device_count: u32 = 0;
        let mut success = false;
        let capture_count_status;
        unsafe {
            capture_count_status = AudioObjectGetPropertyDataSize(
                kAudioObjectSystemObject as AudioObjectID,
                NonNull::new_unchecked(&audio_devices_count_address as *const _ as *mut _),
                0,
                null(),
                NonNull::new_unchecked(&mut device_count as *mut _));
            if capture_count_status == 0 {
                success = true;
            }
        }

        if success {
            let mut device_details: Vec<AudioObjectID> = Vec::with_capacity(device_count as usize);

            unsafe {
                let capture_id_status = AudioObjectGetPropertyData(
                    kAudioObjectSystemObject as AudioObjectID,
                    NonNull::new_unchecked(&audio_devices_count_address as *const _ as * mut _),
                    0,
                    null(),
                    NonNull::new_unchecked(&device_count as *const _ as *mut _),
                    NonNull::new_unchecked(device_details.as_mut_ptr() as *mut c_void));
                if capture_id_status == 0 {
                    device_details.set_len(device_count as usize);
                } else {
                    return Err(Error::DeviceEnumerationFailed(format!("Failed to capture device ids with status {}", capture_id_status)))
                }
            }
            for device in &device_details {
                if *device != 0 {
                    let name = CoreAudio::get_device_name(*device).unwrap();
                    match CoreAudio::check_device_type(*device) {
                        DeviceType::Input => {
                            // May Add Future Functionality
                        },
                        DeviceType::Output => {
                            devices.push(name);
                        },
                        DeviceType::None => {

                        }
                    }

                }
            }
            Ok(devices)
        } else {
            Err(Error::DeviceEnumerationFailed(format!("Failed to capture device count with status {}", capture_count_status)))
        }
        
    }

    // get_vol() on muted device will return 0 as volume
    fn get_vol() -> Result<f32, Error> {
        let mut vol = 0;
        let captured_device_id = CoreAudio::capture_output_device_id();
        if captured_device_id.is_ok() {
            let device_id = captured_device_id.unwrap();
            let mute_property_address = AudioObjectPropertyAddress {
                    mSelector: kAudioDevicePropertyMute,
                    mScope: kAudioDevicePropertyScopeOutput,
                    mElement: kAudioObjectPropertyElementMain
                };

            // Check if Muted
            let mut mute = 0 as u32;
            let mute_data_size = size_of::<u32>() as u32;
            unsafe {
                let mute_status = AudioObjectGetPropertyData(
                    device_id,
                    NonNull::new_unchecked(&mute_property_address as *const _ as *mut _),
                    0,
                    null(),
                    NonNull::new_unchecked(&mute_data_size as *const _ as *mut _),
                    NonNull::new_unchecked(&mut mute as *mut _ as *mut c_void));
                if mute_status != 0 {
                    debug_eprintln("Failed to get mute status");
                }
            }
            if mute == 0 {
                let device_details = CoreAudio::get_output_device_details(device_id);
                if device_details.is_ok() {
                    let channel_count = device_details.unwrap().mChannelsPerFrame;
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
                                    device_id,
                                    NonNull::new_unchecked(&volume_property_address_channel as *const _ as *mut _),
                                    0,
                                    null(),
                                    NonNull::new_unchecked(&mut volume_data_size as *const _ as *mut _),
                                );
                            if get_volume_data_size_status == 0 {
                                let get_volume_status = AudioObjectGetPropertyData(
                                    device_id,
                                    NonNull::new_unchecked(&volume_property_address_channel as *const _ as *mut _),
                                    0,
                                    null(),
                                    NonNull::new_unchecked(&volume_data_size as *const _ as *mut _),
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
        }
        return Ok(vol as f32 / 100.0)
    }

    fn set_vol(value: f32) -> Result<(), Error> {
        let mut status = None;
        let captured_device_id = CoreAudio::capture_output_device_id();
        if captured_device_id.is_ok() {
            let device_id = captured_device_id.unwrap();
            let device_details = CoreAudio::get_output_device_details(device_id);

            if device_details.is_ok() {
                let channel_count = device_details.unwrap().mChannelsPerFrame;

                let volume = value;
                let volume_data_size = size_of::<f32>() as u32;

                for channel in 0..=channel_count {
                    debug_eprintln(&format!("channel {}", channel));
                    let volume_property_address_channel = AudioObjectPropertyAddress {
                        mSelector: kAudioDevicePropertyVolumeScalar,
                        mScope: kAudioDevicePropertyScopeOutput,
                        mElement: channel,
                    };

                    unsafe {
                        let change_volume_status = AudioObjectSetPropertyData(device_id,
                            NonNull::new_unchecked(&volume_property_address_channel as *const _ as *mut _),
                            0, null(),
                            volume_data_size, NonNull::new_unchecked(&volume as *const _ as *mut _));
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
                if value == 0.0 {
                    let mute_data_size = size_of::<u32>() as u32;
                    let mute = 1 as u32;
                    unsafe {
                        let mute_status = AudioObjectSetPropertyData(device_id,
                            NonNull::new_unchecked(&mute_property_address as *const _ as *mut _),
                            0, null(),
                            mute_data_size, NonNull::new_unchecked(&mute as *const _ as *mut _));
                        if mute_status != 0 {
                            sync_status = false;
                        }
                    }
                } else {
                    for mute in (0..=1 as u32).rev() {
                        let mute_data_size = size_of::<u32>() as u32;
                        unsafe {
                            let mute_status = AudioObjectSetPropertyData(device_id,
                                NonNull::new_unchecked(&mute_property_address as *const _ as *mut _),
                                0, null(),
                                mute_data_size, NonNull::new_unchecked(&mute as *const _ as *mut _));
                            if mute_status != 0 {
                                sync_status = false;
                            }
                        }
                    }
                }
                if status.is_none() {
                    status.replace(sync_status);
                }
            } else {
                status.replace(false);
            }
        }
        if status.unwrap_or(false) {Ok(())} else { Err(Error::Placeholder) }
    }

    fn get_mute() -> Result<bool, Error> {
        let mut mute = 0;
        let captured_device_id = CoreAudio::capture_output_device_id();
        if captured_device_id.is_ok() {
            let device_id = captured_device_id.unwrap();
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
                    NonNull::new_unchecked(&mut mute_data_size as *mut _), NonNull::new_unchecked(&mute as *const _ as *mut _));
                if mute_status != 0{
                    debug_eprintln("failed to gather mute status");
                }
            }
        }
        if mute == 1 { Ok(true) } else { Ok(false) }
    }

    fn set_mute(state: bool) -> Result<(), Error> {
        let mut status = true;
        let captured_device_id = CoreAudio::capture_output_device_id();
        if captured_device_id.is_ok() {
            let device_id = captured_device_id.unwrap();
            let mute_property_address = AudioObjectPropertyAddress {
                        mSelector: kAudioDevicePropertyMute,
                        mScope: kAudioDevicePropertyScopeOutput,
                        mElement: kAudioObjectPropertyElementMain
                    };
            let mute_data_size = size_of::<u32>() as u32;
            let mute = match state {
                true => {
                    1
                },
                false => {
                    0
                }
            };
            unsafe {
                let mute_status = AudioObjectSetPropertyData(device_id,
                    NonNull::new_unchecked(&mute_property_address as *const _ as *mut _),
                    0, null(),
                    mute_data_size, NonNull::new_unchecked(&mute as *const _ as *mut _));
                if mute_status != 0 {
                    status = false;
                }
            }
        }
        if status {
            Ok(())
        } else {
            Err(Error::Placeholder)
        }
    }
}

impl CoreAudio {
    // Attempt to Capture Device ID of Default Audio Output Device
    fn capture_output_device_id() -> Result<u32, VolumeError> {
        let mut output_device_address = AudioObjectPropertyAddress {
                mSelector: kAudioHardwarePropertyDefaultOutputDevice,
                mScope: kAudioObjectPropertyScopeGlobal,
                mElement: kAudioObjectPropertyElementMain,
            };
        let mut device_id: AudioObjectID = 0;
        let mut data_size = size_of::<AudioObjectID>() as u32;
        unsafe {
            let capture_output_status = AudioObjectGetPropertyData(
                kAudioObjectSystemObject as u32,
                NonNull::new_unchecked(&mut output_device_address),
                0,
                null(),
                NonNull::new_unchecked(&mut data_size),
                NonNull::new_unchecked(&mut device_id as *mut _ as *mut c_void),
            );

            if capture_output_status == 0 {
                Ok(device_id)
            } else {
                Err(VolumeError::OutputDeviceCaptureError("CoreAudio backend error".to_string()))
            }
        }

    }
    pub fn get_default_output_dev_name() -> String {
        let mut device_name = String::new();
        let captured_device_id = CoreAudio::capture_output_device_id();
        if let Ok(captured_device_id) = captured_device_id {
            let name = CoreAudio::get_device_name(captured_device_id);
            if name.is_ok() {
                device_name.push_str(&name.unwrap());
            }
        }
        device_name
    }

    pub fn check_device_type(device_id: u32) -> DeviceType {
        let mut dev_type_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreams,
            mScope: kAudioObjectPropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMain,
        };

        let mut stream_count: u32 = 0;
        let mut count_size = size_of::<u32>() as u32;
        let capture_type_status;
        unsafe {
            capture_type_status = AudioObjectGetPropertyData(
                device_id,
                NonNull::new_unchecked(&mut dev_type_address),
                0,
                null(),
                NonNull::new_unchecked(&mut count_size),
                NonNull::new_unchecked(&mut stream_count as *mut _ as *mut c_void));
        }
        if capture_type_status == 0 {
            if stream_count > 0 {
                DeviceType::Output
            } else {
                let input_type_address = AudioObjectPropertyAddress {
                        mSelector: kAudioDevicePropertyStreams,
                        mScope: kAudioObjectPropertyScopeInput,
                        mElement: kAudioObjectPropertyElementMain,
                    };
                let mut in_stream_count: u32 = 0;
                let in_count_size = size_of::<u32>() as u32;
                let capture_in_type_status;
                unsafe {
                    capture_in_type_status = AudioObjectGetPropertyData(
                        device_id,
                        NonNull::new_unchecked(&input_type_address as *const _ as *mut _),
                        0,
                        null(),
                        NonNull::new_unchecked(&in_count_size as *const _ as *mut _),
                        NonNull::new_unchecked(&mut in_stream_count as *mut _ as *mut c_void)
                    );
                }
                if capture_in_type_status == 0 {
                    DeviceType::Input
                } else {
                    DeviceType::None
                }
            }
        } else {
            DeviceType::None
        }
    }

    pub fn get_output_device_details(device_id: u32) -> Result<AudioStreamBasicDescription, VolumeError> {
        let mut property_address = AudioObjectPropertyAddress{
            mSelector: kAudioDevicePropertyStreamFormat,
            mScope: kAudioObjectPropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMain,
        };
        
        let mut details: AudioStreamBasicDescription = AudioStreamBasicDescription {
            mSampleRate: 0.0,
            mFormatID: 0,
            mFormatFlags: 0,
            mBytesPerPacket: 0,
            mFramesPerPacket: 0,
            mBytesPerFrame: 0,
            mChannelsPerFrame: 0,
            mBitsPerChannel: 0,
            mReserved: 0 };
        let mut data_size: u32 = size_of::<AudioStreamBasicDescription>() as u32;

        unsafe {
            let detail_capture_status = AudioObjectGetPropertyData(device_id,
                NonNull::new_unchecked(&mut property_address),
                0,
                null(),
                NonNull::new_unchecked(&mut data_size),
                NonNull::new_unchecked(&mut details as *mut _ as *mut c_void));
            if detail_capture_status == 0 {
                Ok(details)
            } else {
                Err(VolumeError::DeviceDetailsCaptureError("CoreAudio backend error.".to_string()))
            }
        }
    }

    pub fn get_device_name(device_id: u32) -> Result<String, VolumeError> {
        let mut property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyDeviceNameCFString,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };
        let mut name: CFStringRef = null_mut();
        let mut data_size = size_of::<CFStringRef>() as u32;
        unsafe {
            let status = AudioObjectGetPropertyData(
                    device_id,
                    NonNull::new_unchecked(&mut property_address),
                    0,
                    null(),
                    NonNull::new_unchecked(&mut data_size),
                    NonNull::new_unchecked(&mut name as *mut _ as *mut _),
                );
            if status == 0 {
                Ok(CFString::wrap_under_get_rule(name).to_string())
            } else {
                debug_eprintln(&format!("Failed to get device name. Status: {}", status));
                Err(VolumeError::NameCaptureError("CoreAudio backend error".to_string()))
            }
        }
    }

    fn get_hw_name(device_id: u32) -> Result<String, VolumeError> {
        #[cfg(target_os = "macos")]
        {
            use objc2_core_audio::kAudioDevicePropertyDeviceName;

            let mut property_address = AudioObjectPropertyAddress {
                mSelector: kAudioDevicePropertyDeviceName,
                mScope: kAudioObjectPropertyScopeGlobal,
                mElement: kAudioObjectPropertyElementMain,
            };

            let mut data_size = size_of::<u32>() as u32;
            unsafe{
                
                
                let size_status = AudioObjectGetPropertyDataSize(
                        device_id,
                        NonNull::new_unchecked(&mut property_address),
                        0,
                        null(),
                        NonNull::new_unchecked(&mut data_size),
                );
                if size_status == 0 {
                    let mut hw_name= Vec::new();
                    hw_name.resize(data_size as usize, 0);
                    let status = AudioObjectGetPropertyData(
                            device_id,
                            NonNull::new_unchecked(&mut property_address),
                            0,
                            null(),
                            NonNull::new_unchecked(&mut data_size),
                            NonNull::new_unchecked(hw_name.as_mut_ptr() as *mut _),
                        );
                    if status == 0 {
                        match String::from_utf8(hw_name) {
                            Ok(name) => {
                                dbg!(name.clone());
                                Ok(name)
                            },
                            Err(e) => {
                                debug_eprintln(&format!("Failed to get device name. Error: {}", e));
                                Err(VolumeError::NameCaptureError(e.to_string()))
                            }
                        }
                    } else {
                        debug_eprintln(&format!("Failed to get device name. Status: {}", status));
                        Err(VolumeError::NameCaptureError("CoreAudio backend error".to_string()))
                    }
                } else {
                    Err(VolumeError::NameCaptureError("CoreAudio backend error".to_string()))
                }
            }
        }

    }

    fn uid_to_hw_id(uid: String) -> Result<u32, Error> {
        let id_property_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDeviceForUID,
            mScope: kAudioDevicePropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMain,
        };

        let cf_uid = CFString::new(&uid);

        let mut hw_id: u32 = 0;
        let mut data_size = size_of::<u32>() as u32;
        unsafe {
            use std::ptr::{NonNull, null};
            let hw_id_status = AudioObjectGetPropertyData(kAudioObjectSystemObject as u32,
                NonNull::new_unchecked(&id_property_address as *const _ as *mut _),
                size_of::<String>() as u32, cf_uid.as_concrete_TypeRef() as *const c_void,
                NonNull::new_unchecked(&mut data_size as *mut _ ), NonNull::new_unchecked(&hw_id as *const _ as *mut _));
            if hw_id_status != 0 {
                return Err(Error::Placeholder);
            }
        }
        Ok(hw_id)
    }

}

pub struct CoreAudioDevice {
    device_id: u32,
    device_uid: String,
}

impl CoreAudioDevice {
    pub fn get_device_hw_id(&self) -> u32 {
        self.device_id
    }

}


