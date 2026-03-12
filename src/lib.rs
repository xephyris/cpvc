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
//!     let current_volume: u8 = cpvc::get_system_volume();
//! 
//!     // Set system volume for default output in %
//!     let volume: u8 = 32;
//!     let success = cpvc::set_system_volume(volume);
//!     
//!     // Mute default output
//!     let success = cpvc::set_system_volume(0);
//! }
//! ```

use crate::error::Error;

pub mod legacy;

// Functionality may be added in future versions
pub mod device;
// pub mod cpal;

pub mod coreaudio;
pub mod wasapi;
#[cfg(target_os = "linux")]
pub mod pulseaudio;

pub mod error;
pub mod safe;

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
pub fn get_sound_devices() -> Vec<String> {
    safe::get_sound_devices().unwrap_or(Vec::new())
}

/// Gathers the current volume in percent of the default output device
pub fn get_system_volume() -> u8 {
    safe::get_system_volume().unwrap()

}


/// Sets the current volume in percent of the default output device
/// ## On macOS
/// `cpvc` needs to mute and unmute the audio device to get the hardware device volume to sync 
pub fn set_system_volume(percent: u8) -> bool {
    if let Ok(_) = safe::set_system_volume(percent) {
        true
    } else {
        false
    }
}

pub fn set_mute(mute: bool) -> bool {
    if let Ok(_) = safe::set_mute(mute) {
        true
    } else {
        false
    }
}

pub fn get_mute() -> bool {
    safe::get_mute().unwrap_or(false)
}

// TODO add get_default_output_device() function back

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
            let device = pulseaudio::device::PulseAudioDevice::from_id("".to_string()).unwrap();
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
            let devices = pulseaudio::PulseAudio::get_device_identifiers().unwrap();
            dbg!(&devices);
            for (device_id, name) in devices {
                println!("{}", format!("DEVICE STR {}, NAME: {}", unsafe {device_id.to_string()}, name));
            }
        }
        assert!(false);
    }

    #[test]
    fn set_sound_test() {
        dbg!(set_system_volume(2));
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
