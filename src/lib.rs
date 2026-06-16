//! `cpvc` is a simple cross-platform audio control crate
//! 
//! Currently, cpvc supports the following platforms
//! * macOS
//! * Windows
//! * Linux (`pulse_audio` only)
//!
//! To access platform specfic modules, you need to be on the specific OS
//! This functionality may change in future versions
//!  * macOS -> `coreaudio`
//!  * windows -> `wasapi`
//!  * linux -> `pulseaudio`
//! 
//! # Controls Example
//!
//! ```rust,
//! fn main() {
//!     
//!     // Gets current system output device names (human readable ones)
//!     let devices: Vec<String> = cpvc::get_sound_devices();
//! 
//!     // Get current system volume for default output in %
//!     let current_volume: f32 = cpvc::get_system_volume();
//! 
//!     // Get if the default audio device is muted
//!     let mute_status = cpvc::get_mute();
//! 
//!     // Set system volume for default output in %
//!     let volume: f32 = 0.32;
//!     let success = cpvc::set_system_volume(volume);
//!     
//!     // Mute default output
//!     let success = cpvc::set_mute(true);
//! }
//! ```

use crate::{device::{Device, DeviceTrait}, error::Error::{self, PlatformUnsupported}};

pub mod legacy;

// Functionality may be added in future versions
pub mod device;

#[cfg(feature = "cpal")]
pub mod cpal;

pub mod coreaudio;
pub mod wasapi;
pub mod pulseaudio;

pub mod error;

#[cfg(feature = "debug")]
fn debug_eprintln(message: &str){
    eprintln!("{}", message);
}

#[cfg(feature = "debug")]
fn debug_println(message: &str) {
    println!("{}", message);
}

#[cfg(not(feature = "debug"))]
fn debug_eprintln(_: &str){

}

#[cfg(not(feature = "debug"))]
fn debug_println(_: &str) {

}

pub trait VolumeControl {
    fn get_sound_devices() -> Result<Vec<String>, Error>;

    fn get_vol() -> Result<f32, Error>;

    fn set_vol(value: f32) -> Result<(), Error>;

    fn get_mute() -> Result<bool, Error>;

    fn set_mute(state: bool) -> Result<(), Error>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum DeviceType {
    Input,
    Output,
    None,
}

/// Gathers the human readable device name of each output device detected
pub fn try_get_sound_devices() -> Result<Vec<String>, Error> {
    #[cfg(target_os="macos")] {
        return coreaudio::get_sound_devices();
    }
    #[cfg(target_os="windows")] {
        return wasapi::get_sound_devices();
    }
    #[cfg(target_os="linux")] {
        return pulseaudio::get_sound_devices();
    }
    Err(Error::PlatformUnsupported)
}

pub fn get_sound_devices() -> Vec<String> {
    try_get_sound_devices().unwrap_or(Vec::new())
}

/// Gathers the current volume in percent of the default output device
pub fn try_get_system_volume() -> Result<f32, Error> {
    #[allow(unused_assignments)]
    #[cfg(target_os="macos")] {
       return coreaudio::get_vol();
    }
    #[cfg(target_os="windows")] {
        return wasapi::get_vol();
    }
    #[cfg(target_os="linux")] {
        return pulseaudio::get_vol();
    }
    Err(PlatformUnsupported)
}

pub fn get_system_volume() -> f32 {
    try_get_system_volume().unwrap_or(0.0)
}

pub fn get_system_volume_u8() -> u8 {
    (get_system_volume() * 100.0) as u8
}

/// Sets the current volume in percent of the default output device
/// ## On macOS
/// `cpvc` needs to mute and unmute the audio device to get the hardware device volume to sync 
pub fn try_set_system_volume(percent: f32) -> Result<bool, Error> {
    #[cfg(target_os="macos")] {
        coreaudio::set_vol(percent)?;
        return Ok(true);
    }
    #[cfg(target_os="windows")] {
        wasapi::set_vol(percent)?;
        return Ok(true);
    }
    #[cfg(target_os="linux")] {
        pulseaudio::set_vol(percent)?;
        return Ok(true);
    }
    Err(PlatformUnsupported)
}

pub fn set_system_volume(percent: f32) -> bool {
    if let Ok(status) = try_set_system_volume(percent) {
        return status
    } else {
        false
    }
}

pub fn set_system_volume_u8(percent: u8) -> bool {
    set_system_volume(percent as f32 / 100.0)
}

pub fn try_set_mute(mute: bool) -> Result<bool, Error> {
    #[cfg(target_os="macos")] {
        coreaudio::set_mute(mute)?.map();
        return Ok(true);
    }
    #[cfg(target_os="windows")]
    {
        wasapi::set_mute(mute)?;
        return Ok(true);
    }
    #[cfg(target_os="linux")] {
        pulseaudio::set_mute(mute)?;
        return Ok(true);
    }
    Err(PlatformUnsupported)
}

pub fn set_mute(mute: bool) -> bool {
    if let Ok(status) = try_set_mute(mute) {
        status
    } else {
        false
    }
}

pub fn try_get_mute() -> Result<bool, Error> {
    #[cfg(target_os="macos")] {
        return coreaudio::get_mute();
    }
    #[cfg(target_os="windows")] {
        return wasapi::get_mute();
    }
    #[cfg(target_os="linux")] {
        return pulseaudio::get_mute();
    }
    Err(PlatformUnsupported)
}

pub fn get_mute() -> bool {
    if let Ok(status) = try_get_mute() {
        status
    } else {
        false
    }
}

// TODO add get_default_output_device() function back

pub fn get_default_output_device() -> Result<Device, Error>{
    #[cfg(target_os="macos")] {
        use crate::device::DeviceTrait;
        return Device::from_uid(coreaudio::get_default_output_device()?.get_uid()?)
    }
    #[cfg(target_os="windows")] {
        return Device::from_uid(wasapi::get_default_output_device()?.get_device_uid()?)
    }
    #[cfg(target_os="linux")] {
        return Device::from_uid(pulseaudio::get_default_output_dev()?.get_uid()?)
    }
    Err(Error::PlatformUnsupported)
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;

    #[test]

    fn sound_devices() {
        dbg!(get_sound_devices());
        assert!(false);
    }

    #[test]
    // #[ignore]
    // Change HW ID before running
    fn test_non_default_device() {
        #[cfg(target_os="macos")] {
            use crate::device::DeviceTrait;

            let device = coreaudio::device::CoreAudioDevice::from_hw_id(0).unwrap();
            dbg!(device.get_device_hw_id());
            dbg!(device.get_name());
            dbg!(device.set_mute(true));
            dbg!(device.get_vol());
            dbg!(device.set_vol(0.1));
        }
        
        #[cfg(target_os="windows")] {
            use crate::device::DeviceTrait;

            let device = wasapi::device::WASAPIDevice::from_uid("".to_string()).unwrap();
            dbg!(device.get_device_uid());
            dbg!(device.get_name());
            dbg!(device.set_mute(false));
            dbg!(device.get_vol());
            dbg!(device.set_vol(0.1));
        }
        
        #[cfg(target_os="linux")] {
            use crate::device::DeviceTrait;

            let device = pulseaudio::device::PulseAudioDevice::from_uid("".to_string()).unwrap();
            dbg!(device.get_device_str());
            dbg!(device.get_name());
            dbg!(device.set_mute(false));
            dbg!(device.get_vol());
            dbg!(device.set_vol(0.1));
        }

        assert!(false);

    }

    #[test]
    fn get_device_idents() {
        
        #[cfg(target_os="windows")] {
            let devices = wasapi::get_device_identifiers().unwrap();
            dbg!(&devices);
            for (device_id, name) in devices {
                println!("{}", format!("DEVICE ID {}, NAME: {}", unsafe {device_id.to_string()}, name));
            }
        }
        #[cfg(target_os="linux")] {
            let devices = pulseaudio::get_device_identifiers().unwrap();
            dbg!(&devices);
            for (device_id, name) in devices {
                println!("{}", format!("DEVICE STR {}, NAME: {}", unsafe {device_id.to_string()}, name));
            }
        }
        assert!(false);
    }

    #[test]
    fn set_sound_test() {
        dbg!(set_system_volume_u8(2));
        assert!(false);
    }

    #[test]
    fn get_sound_test() {
        dbg!(get_system_volume());
        assert!(false);
    }

    #[test]
    fn set_mute_test() {
        dbg!(set_mute(true));
        dbg!(get_system_volume());
        assert!(false);
    }

    #[test]
    fn get_mute_status() {
        dbg!(get_mute());
        dbg!(get_system_volume());
        assert!(false);
    }

    #[cfg(target_os="linux")]
    #[test]
    fn test_alsa_get_device() {
        dbg!(pulseaudio::convert_alsa_id("2".to_string(), "0".to_string()));
        assert!(false)
    }

    #[cfg(target_os="macos")] 
    #[test]
    fn get_dev_hw_name() {
        // dbg!(get_hw_name(capture_output_device_id().unwrap()));
        assert!(false)
    }


    // #[cfg(target_os="macos")]
    // #[test]
    // #[ignore]
    // fn get_device_details() {
    //     println!("{}", get_default_output_dev());
    //     assert!(false);
    // }

    // #[cfg(target_os="linux")]
    // #[test]
    // fn get_pulse_output_devices() {
    //     println!("{}", get_default_output_dev());
    //     assert!(false);
    // }
}
