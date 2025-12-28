use objc2_core_audio::kAudioHardwarePropertyDeviceForUID;

use crate::{DeviceType, VolumeControl, VolumeError, debug_eprintln, error::Error, legacy::get_default_output_dev};

pub mod device;

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
        if let Ok(identifiers) = CoreAudio::get_device_identifiers() {
            Ok(identifiers.into_iter().map(|(_id, name)| name).collect())
        } else {
            Err(Error::Placeholder)
        }
    }

    fn get_vol() -> Result<f32, Error> {
        let output_dev = CoreAudio::capture_output_device()?;
        output_dev.get_vol()
    }

    fn set_vol(vol: f32) -> Result<(), Error> {
        let output_dev = CoreAudio::capture_output_device()?;
        output_dev.set_vol(vol)
    }

    fn get_mute() -> Result<bool, Error> {
        let output_dev = CoreAudio::capture_output_device()?;
        output_dev.get_mute()
    }

    fn set_mute(mute: bool) -> Result<(), Error> {
        let output_dev = CoreAudio::capture_output_device()?;
        output_dev.set_mute(mute)
    }
}

impl CoreAudio {

    fn get_device_identifiers() -> Result<Vec<(u32, String)>, Error> {
        let mut devices:Vec<(u32, String)> = Vec::new();
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
                            devices.push((*device, name));
                        },
                        DeviceType::None => {}
                    }
                }
            }
            Ok(devices)
        } else {
            Err(Error::DeviceEnumerationFailed(format!("Failed to capture device count with status {}", capture_count_status)))
        }
        
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

    pub fn get_device_name(device_id: u32) -> Result<String, Error> {
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
                Err(Error::DeviceEnumerationFailed("Name Capture failed CoreAudio backend error".to_string()))
            }
        }
    }

    pub fn get_output_device_details(device_id: u32) -> Result<AudioStreamBasicDescription, Error> {
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
                Err(Error::DeviceEnumerationFailed("CoreAudio backend error.".to_string()))
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

    // Attempt to Capture Device ID of Default Audio Output Device
    fn capture_output_device() -> Result<device::CoreAudioDevice, Error> {
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
                Ok(device::CoreAudioDevice::from_hw_id(device_id))?
            } else {
                Err(Error::DeviceEnumerationFailed("CoreAudio backend error".to_string()))
            }
        }

    }

}
