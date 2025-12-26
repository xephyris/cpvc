#[cfg(target_os="macos")]
use {
    objc2_core_audio::{
        kAudioDevicePropertyDeviceNameCFString, kAudioDevicePropertyMute, kAudioDevicePropertyScopeOutput, kAudioDevicePropertyStreamFormat, kAudioDevicePropertyStreams, kAudioDevicePropertyVolumeScalar, kAudioHardwarePropertyDefaultOutputDevice, kAudioHardwarePropertyDevices, kAudioObjectPropertyElementMain, kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyScopeInput, kAudioObjectPropertyScopeOutput, kAudioObjectSystemObject, AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize, AudioObjectID, AudioObjectPropertyAddress, AudioObjectSetPropertyData
    }, objc2_core_audio_types::AudioStreamBasicDescription, 
};

use std::{collections::HashMap, ffi::c_void, ptr::{null, NonNull}};


// TODO Change to iterator for AudioDevice?
pub fn scan_devices() -> HashMap<String, u32> {
    let mut devices:HashMap<String, u32> = HashMap::new();
    #[cfg(target_os="macos")] {
        let audio_devices_count_address =  AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDevices,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        };

        let mut device_count: u32 = 0;
        let mut success = false;

        unsafe {
            let capture_count_status = AudioObjectGetPropertyDataSize(
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
                }
            }
            for device in &device_details {
                if *device != 0 {
                    use crate::{coreaudio::CoreAudio, DeviceType};

                    let name = CoreAudio::get_device_name(*device).unwrap();
                    match CoreAudio::check_device_type(*device) {
                        DeviceType::Input => {
                            // May Add Future Functionality
                        },
                        DeviceType::Output => {
                            devices.insert(name, *device);
                        },
                        DeviceType::None => {

                        }
                    }

                }
            }
        }
    }
    devices
}