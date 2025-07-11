
use std::process::Command;

#[cfg(target_os="macos")] 
use {
    std::ffi::c_void,
    std::ptr::{null, null_mut},
    std::mem::{size_of},
    std::ptr::NonNull, 
    core_foundation::{base::TCFType, string::{CFString, CFStringRef}},
    objc2_core_audio_types::{AudioStreamBasicDescription},
    objc2_core_audio::{
        AudioObjectGetPropertyData, AudioObjectSetPropertyData, AudioObjectID, AudioObjectPropertyAddress,
        kAudioHardwarePropertyDefaultOutputDevice, kAudioObjectSystemObject,
        kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyElementMain,
        kAudioDevicePropertyScopeOutput, kAudioDevicePropertyMute,
        kAudioDevicePropertyVolumeScalar, kAudioDevicePropertyDeviceNameCFString,
        kAudioDevicePropertyStreamFormat, kAudioObjectPropertyScopeOutput,
    },
};


#[cfg(target_os="linux")]
// use alsa::{card, ctl, pcm, mixer::{SelemId, Mixer, SelemChannelId}};
use {
    alsa::{ctl, mixer::{SelemId, Mixer, SelemChannelId}}
};

pub mod command;

use std::env;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Error {
    OutputDeviceCaptureError,
    DeviceDetailsCaptureError,

}

pub fn get_sound_devices() -> Vec<String> {
    let mut devices:Vec<String> = Vec::new();
    #[cfg(target_os="macos")] {
        
    }
    #[cfg(target_os="windows")] {

    }
    #[cfg(target_os="linux")] {
        // ALSA cannot detect cards that show up in PipeWire 
        // This function records the HW audio devices that are directly connected to the device
        // Bluetooth devices are handled by PipeWire so they will not appear
        let control = ctl::Ctl::new("pipewire", false);
        let mut name = String::from("");
        match control {
            Ok(_) => {
                name.push_str("pipewire");
            },
            Err(_) => {
                let control = Mixer::new("pulse", false);
                match control {
                    Ok(_) => {
                        name.push_str("pulse");
                    },
                    Err(_) => {
                        eprintln!("CPVC only supports PipeWire and PulseAudio at the moment, please check back to see if your framework is supported");
                    }
                }
               
            }

        }

        let mut count = 0;
        let mut audio_devices = Vec::new();
        while let Ok(cdev) = ctl::Ctl::new(&format!("hw:{}", count), false) {
            audio_devices.push(cdev);
            count += 1;
        } 

        devices.append(&mut audio_devices.into_iter().map(|ctl| ctl.card_info().unwrap().get_mixername().unwrap().to_owned()).collect::<Vec<String>>());
    }
    devices
}

pub fn get_system_volume() -> u8 {
    #[allow(unused_assignments)]
    let mut vol = 0;
    #[cfg(target_os="macos")] {
        let output = Command::new("osascript").arg("-e").arg("return output volume of (get volume settings)").output().expect("Are you running on MacOS?");
        let out = String::from_utf8_lossy(&output.stdout).to_string().trim().to_owned();
        vol = out.parse::<u8>().unwrap_or(0);
    }
    #[cfg(target_os="linux")] {
        let mixer = Mixer::new("pipewire", false);
        let mut name = String::from("");
        match mixer {
            Ok(_) => {
                name = String::from("pipewire");
            },
            Err(_) => {
                let mixer = Mixer::new("pulse", false);
                match mixer {
                    Ok(_) => {
                        name = String::from("pulse");
                    },
                    Err(_) => {
                        eprintln!("CPVC only supports PipeWire and PulseAudio at the moment, please check back to see if your framework is supported");
                    }
                }
               
            }

        }
        
        if name != "" {
            let mixer = Mixer::new(&name, false).unwrap();
            let id = SelemId::new("Master", 0);
            let selem = mixer.find_selem(&id).unwrap();
            let mut sum = 0;
            let mut count = 0;
            let mut citer =  SelemChannelId::all().iter();
            let factor = selem.get_playback_volume_range().1 - selem.get_playback_volume_range().0;
            while let Some(channel) = citer.next(){
                if selem.has_playback_channel(*channel) {
                    sum += selem.get_playback_volume(*channel).unwrap_or_default();
                    count += 1;
                }
            }
            vol = (sum as f32 / count as f32 / factor as f32 * 100_f32) as u8;
        }
    }
    vol
    
}



pub fn set_system_volume(percent: u8) -> bool {
    #[allow(unused_assignments)]
    let mut success = true;
    #[cfg(target_os="macos")]{
        let captured_device_id = capture_output_device_id();
        if captured_device_id.is_ok() {
            let device_id = captured_device_id.unwrap();
            
            let volume_property_address_left = AudioObjectPropertyAddress {
                mSelector: kAudioDevicePropertyVolumeScalar,
                mScope: kAudioDevicePropertyScopeOutput,
                mElement: 1 
            };
            let volume_property_address_right = AudioObjectPropertyAddress {
                mSelector: kAudioDevicePropertyVolumeScalar,
                mScope: kAudioDevicePropertyScopeOutput,
                mElement: 2 
            };

            let mute_property_address = AudioObjectPropertyAddress {
                mSelector: kAudioDevicePropertyMute,
                mScope: kAudioDevicePropertyScopeOutput,
                mElement: kAudioObjectPropertyElementMain
            };


            let volume = percent as f32 / 100 as f32;
            let volume_data_size = size_of::<f32>() as u32;

            let mute: u32 = 1;
            let unmute: u32 = 0;
            let mute_data_size = size_of::<u32>() as u32;
            let unmute_data_size = size_of::<u32>() as u32;

               
            unsafe {
                let _left_result = AudioObjectSetPropertyData(device_id,
                                NonNull::new_unchecked(&volume_property_address_left as *const _ as *mut _),
                                0, null(),
                                volume_data_size, NonNull::new_unchecked(&volume as *const _ as *mut _));
                let _right_result = AudioObjectSetPropertyData(device_id,
                                NonNull::new_unchecked(&volume_property_address_right as *const _ as *mut _),
                                0, null(),
                                volume_data_size, NonNull::new_unchecked(&volume as *const _ as *mut _));

                let _mute = AudioObjectSetPropertyData(device_id,
                                NonNull::new_unchecked(&mute_property_address as *const _ as *mut _),
                                0, null(),
                                mute_data_size, NonNull::new_unchecked(&mute as *const _ as *mut _));
                let _unmute = AudioObjectSetPropertyData(device_id,
                                NonNull::new_unchecked(&mute_property_address as *const _ as *mut _),
                                0, null(),
                                unmute_data_size, NonNull::new_unchecked(&unmute as *const _ as *mut _));
            }
        }
    }
    #[cfg(target_os="linux")] {
        let mixer = Mixer::new("pipewire", false);
        let mut name = String::from("");
        match mixer {
            Ok(_) => {
                name = String::from("pipewire");
            },
            Err(_) => {
                let mixer = Mixer::new("pulse", false);
                match mixer {
                    Ok(_) => {
                        name = String::from("pulse");
                    },
                    Err(_) => {
                        eprintln!("CPVC only supports PipeWire and PulseAudio at the moment, please check back to see if your framework is supported");
                    }
                }
               
            }
        }

        if name != "" {
            let mixer = Mixer::new(&name, false).unwrap();
            let id = SelemId::new("Master", 0);
            let selem = mixer.find_selem(&id).unwrap();
            let mut citer =  SelemChannelId::all().iter();
            let factor = selem.get_playback_volume_range().1 - selem.get_playback_volume_range().0;
            
            while let Some(channel) = citer.next(){
                if selem.has_playback_channel(*channel) {
                    selem.set_playback_volume(*channel, percent as i64 * factor / 100).unwrap_or_else(|_| success = false);
                }
            }
        }
    }
    success
}

pub fn get_default_output_dev() -> String {
    let mut device_name = String::new();
    #[cfg(target_os = "macos")]
    {
        let captured_device_id = capture_output_device_id();
        if captured_device_id.is_ok() {

            let device_id = captured_device_id.unwrap();
            let property_address = AudioObjectPropertyAddress {
                mSelector: kAudioDevicePropertyDeviceNameCFString,
                mScope: kAudioObjectPropertyScopeGlobal,
                mElement: kAudioObjectPropertyElementMain,
            };
            unsafe{
                let mut name: CFStringRef = null_mut();
                let data_size = size_of::<CFStringRef>() as u32;
                let status = AudioObjectGetPropertyData(
                        device_id,
                        NonNull::new_unchecked(&property_address as *const _ as *mut _),
                        0,
                        null(),
                        NonNull::new_unchecked(&data_size as *const _ as *mut _),
                        NonNull::new_unchecked(&mut name as *mut _ as *mut _),
                    );
                if status == 0 {
                    device_name.push_str(&CFString::wrap_under_get_rule(name).to_string());
                } else {
                    eprintln!("Failed to get device name. Status: {}", status);
                }
            }
        }
    }
    device_name
}

#[cfg(target_os = "macos")]
fn capture_output_device_id() -> Result<u32, Error> {
    unsafe {
        // Attempt to Capture Device ID of Default Audio Output Device
        let output_device_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDefaultOutputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        let mut device_id: AudioObjectID = 0;
        let mut data_size = size_of::<AudioObjectID>() as u32;

        let capture_output_status = AudioObjectGetPropertyData(
            kAudioObjectSystemObject as u32,
            NonNull::new_unchecked(&output_device_address as *const _ as *mut _),
            0,
            null(),
            NonNull::new_unchecked(&mut data_size),
            NonNull::new_unchecked(&mut device_id as *mut _ as *mut c_void),
        );
        
        if capture_output_status == 0 {
            Ok(device_id)
        } else {
            Err(Error::OutputDeviceCaptureError)
        }
    }

}

#[cfg(target_os="macos")] 
fn get_output_device_details(device_id: u32) -> Result<AudioStreamBasicDescription, Error> {
    let property_address = AudioObjectPropertyAddress{
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
    let data_size = size_of::<AudioStreamBasicDescription>();

    unsafe {
        let detail_capture_status = AudioObjectGetPropertyData(device_id, 
            NonNull::new_unchecked(&property_address as *const _ as *mut _ ), 
            0, 
            null(), 
            NonNull::new_unchecked(&data_size as *const _ as *mut _), 
            NonNull::new_unchecked(&mut details as *mut _ as *mut c_void));
        if detail_capture_status == 0 {
            Ok(details)
        } else {
            Err(Error::DeviceDetailsCaptureError)
        }
    }


}


pub fn get_os() -> String {
    println!("{}", env::consts::OS);
    env::consts::OS.to_string()
}


#[cfg(test)]
mod tests {
    use std::env;
    use super::*;

    #[test]
    #[ignore]
    fn test_os() {
        println!("{}", get_os());
        assert_eq!(env::consts::OS, get_os());
    }

    #[test] 
    
    fn sound_devices() {
        get_sound_devices();
        // get_default_output_dev();
        assert_eq!(false, true);
    }

    #[test]
    #[ignore]
    fn current_output() {
        // assert!(set_system_volume(24));
    }

    #[test] 
    #[ignore]
    fn sound_devices_cmd() {
        assert_eq!(false, true);
    }

    #[test]
    #[ignore]
    fn current_output_cmd() {
        assert!(command::set_system_volume_command(24));
    }

    #[test]
    fn get_device_details() {
        println!("{}", get_default_output_dev());
        get_output_device_details(capture_output_device_id().unwrap());
        set_system_volume(20);
        assert!(false);
    }


}