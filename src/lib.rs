//! `cpvc` is a simple cross-platform audio control crate
//! 
//! Currently, cpvc supports the following platforms
//! * macOS
//! * Windows
//! * Linux (`pulse_audio` only)
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

// TODO change unwrap() to better error handling method

use std::env;

use crate::error::Error;



#[cfg(target_os="linux")]
// use alsa::{card, ctl, pcm, mixer::{SelemId, Mixer, SelemChannelId}};
use {
    alsa::{ctl, mixer::{SelemId, Mixer, SelemChannelId}},
    libpulse_binding::{
        context::Context, 
        mainloop::standard::Mainloop,
        proplist::Proplist
    },
};

pub mod command;
pub mod legacy;
pub mod device;
pub mod scan;


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
enum DeviceType {
    Input,
    Output,
    None,
}

#[derive(Debug, Clone, PartialEq)]
enum VolumeError {
    OutputDeviceCaptureError(String),
    DeviceDetailsCaptureError(String),
    NameCaptureError(String),
}



/// Gathers the human readable device name of each output device detected
pub fn get_sound_devices() -> Vec<String> {
    let mut devices:Vec<String> = Vec::new();
    #[cfg(target_os="macos")] {
        devices = coreaudio::get_sound_devices().unwrap();
    }
    #[cfg(target_os="windows")] {
        unsafe {
            use windows::Win32::Media::Audio::{eRender, DEVICE_STATE_ACTIVE};
            use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
            use windows::Win32::System::Com::STGM_READ;

            let enumerator = get_enumerator();
            let device_col = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE).unwrap();
            let dev_count = device_col.GetCount().unwrap();
            for device_id in 0..dev_count{
                let device = device_col.Item(device_id).unwrap();
                let result = device.OpenPropertyStore(STGM_READ);
                match result {
                    Ok(properties) => {
                        let name = properties.GetValue(&PKEY_Device_FriendlyName).unwrap();
                        devices.push(name.to_string());
                        // dbg!(properties.GetValue(&PKEY_Device_FriendlyName));
                    },
                    Err(error) => {
                        panic!("{}", error);
                    }
                }
            }

        }
    }
    #[cfg(target_os="linux")] {
        use std::sync::{Arc, Mutex};

        let device_list = Arc::new(Mutex::new(Vec::new()));
        let mut mainloop = Mainloop::new().expect("Failed to create mainloop");
        let proplist = Proplist::new().unwrap();
        let mut context = Context::new_with_proplist(&mainloop, "CPVC", &proplist)
            .expect("Failed to create connection context");


        context.connect(None, libpulse_binding::context::FlagSet::NOFLAGS, None)
            .expect("Failed to connect context");

        loop {
            match context.get_state() {
                libpulse_binding::context::State::Ready => break,
                libpulse_binding::context::State::Failed | libpulse_binding::context::State::Terminated => {
                    panic!("Context failed or terminated");
                }
                _ => {
                    mainloop.iterate(false);
                }
            }
        }
        let clone = Arc::clone(&device_list);

        let op = context.introspect().get_sink_info_list(move |info | {
                match info {
                    libpulse_binding::callbacks::ListResult::Item(device) => {
                        clone.lock().unwrap().push(device.description.as_ref().unwrap().to_string());
                    },
                    libpulse_binding::callbacks::ListResult::End => {
                        debug_println("Devices finished");
                    },
                    libpulse_binding::callbacks::ListResult::Error => {
                        debug_eprintln("error gathering device information");
                    },
                }
            });
        
        while op.get_state() == libpulse_binding::operation::State::Running {
            mainloop.iterate(false);
        }

        mainloop.quit(libpulse_binding::def::Retval(0));

        devices.append(&mut device_list.lock().unwrap());
    }
    devices
}

/// Gathers the current volume in percent of the default output device
pub fn get_system_volume() -> u8 {
    #[allow(unused_assignments)]
    let mut vol: u8 = 0;
    #[cfg(target_os="macos")] {
       vol = (coreaudio::get_vol().unwrap() * 100.0) as u8;
    }
    #[cfg(target_os="windows")] {
        use windows::Win32::System::Com::CLSCTX_ALL;
        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;

        let device = get_default_output_device();
        unsafe {
            let volume_controls = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).unwrap();
            if volume_controls.GetMute().unwrap().into() {
                vol = 0;
            } else {
                let channel_count = volume_controls.GetChannelCount().unwrap();
                let mut total_volumes = 0.0;
                for channel in 0..channel_count {
                    total_volumes += volume_controls.GetChannelVolumeLevelScalar(channel).unwrap();
                }
                total_volumes *= 100.0;
                vol = (total_volumes / channel_count as f32).round() as u8;
            }

            // dbg!(volume_controls);
        }
    }
    #[cfg(target_os="linux")] {
        use std::sync::{Arc, Mutex};

        let volume = Arc::new(Mutex::new(vol));
        let clone = Arc::clone(&volume);

        let mut mainloop = Mainloop::new().expect("Failed to create mainloop");
        let proplist = Proplist::new().unwrap();
        let mut context = Context::new_with_proplist(&mainloop, "CPVC", &proplist)
            .expect("Failed to create connection context");
        
        context.connect(None, libpulse_binding::context::FlagSet::NOFLAGS, None)
            .expect("Failed to connect context");

        loop {
            match context.get_state() {
                libpulse_binding::context::State::Ready => break,
                libpulse_binding::context::State::Failed | libpulse_binding::context::State::Terminated => {
                    panic!("Context failed or terminated");
                }
                _ => {
                    mainloop.iterate(false);
                }
            }
        }
        let op = context.introspect().get_sink_info_list( move |info | {
                match info {
                    libpulse_binding::callbacks::ListResult::Item(device) => {
                        if device.mute {
                            *clone.lock().unwrap() = 0;
                        } else {
                            let mut vol_str = device.volume.avg().print().trim().to_string();
                            vol_str.remove(vol_str.len() - 1);
                            match vol_str.parse::<u8>() {
                                Ok(vol) => {
                                    *clone.lock().unwrap() = vol;
                                },
                                Err(err) => {
                                    debug_eprintln(&format!("Failed to parse volume string {}", err));
                                }
                            }
                        }
                    },
                    libpulse_binding::callbacks::ListResult::End => {
                        debug_println("Devices finished")
                    },
                    libpulse_binding::callbacks::ListResult::Error => {
                        debug_eprintln("error gathering device information");
                    },
                }
            });
        
        while op.get_state() == libpulse_binding::operation::State::Running {
            mainloop.iterate(false);
        }

        mainloop.quit(libpulse_binding::def::Retval(0));

        vol = *volume.lock().unwrap();
    }
    vol

}


/// Sets the current volume in percent of the default output device
/// ## On macOS
/// `cpvc` needs to mute and unmute the audio device to get the hardware device volume to sync 
pub fn set_system_volume(percent: u8) -> bool {
    #[allow(unused_assignments)]
    let mut success = None;
    #[cfg(target_os="macos")] {
        if let Ok(_) = coreaudio::set_vol(percent as f32 / 100.0) {
            success = Some(true)
        } else {
            success.replace(false);
        }
    }
    #[cfg(target_os="windows")] {
        use windows::Win32::System::Com::CLSCTX_ALL;
        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
        use std::ptr;

        let device = get_default_output_device();
        unsafe {
            let volume_controls = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).unwrap();
            if volume_controls.GetMute().unwrap().into() {
                volume_controls.SetMute(false, ptr::null()).unwrap();
            }

            let channel_count = volume_controls.GetChannelCount().unwrap();
            for channel in 0..channel_count {
                volume_controls.SetChannelVolumeLevelScalar(channel, percent as f32 / 100.0, ptr::null()).unwrap();
            }

            if percent == 0 {
                volume_controls.SetMute(true, ptr::null()).unwrap();
            }
        }
        success.replace(true);
    }
    #[cfg(target_os="linux")] {

        use std::sync::{Arc, Mutex};
        
        let vol_channels = Arc::new(Mutex::new(None));
        let clone = Arc::clone(&vol_channels);

        let mut mainloop = Mainloop::new().expect("Failed to create mainloop");
        let proplist = Proplist::new().unwrap();
        let mut context = Context::new_with_proplist(&mainloop, "CPVC", &proplist)
            .expect("Failed to create connection context");
        
        context.connect(None, libpulse_binding::context::FlagSet::NOFLAGS, None)
            .expect("Failed to connect context");

        loop {
            match context.get_state() {
                libpulse_binding::context::State::Ready => break,
                libpulse_binding::context::State::Failed | libpulse_binding::context::State::Terminated => {
                    panic!("Context failed or terminated");
                }
                _ => {
                    mainloop.iterate(false);
                }
            }
        }
        let op = context.introspect().get_sink_info_list( move |info | {
                match info {
                    libpulse_binding::callbacks::ListResult::Item(device) => {
                        if let Some(_) = device.active_port {
                            use libpulse_binding::volume::{Volume};
                            use libpulse_sys::volume::PA_VOLUME_NORM;
                            let vol = Volume(percent as u32 * PA_VOLUME_NORM / 100);
                            let mut channel_vols = device.volume;
                            channel_vols.set(device.sample_spec.channels, vol.into());
                            clone.lock().unwrap().replace((device.index, channel_vols));
                        }
                    },
                    libpulse_binding::callbacks::ListResult::End => {
                        debug_println("channel volume aquired");
                    },
                    libpulse_binding::callbacks::ListResult::Error => {
                        debug_eprintln("error gathering device information");    
                    },
                }
            });

        
        while op.get_state() == libpulse_binding::operation::State::Running {
            mainloop.iterate(false);
        }

        if let Some((index, volume)) = vol_channels.lock().unwrap().take() {
            let vol_runner;
            if percent == 0 {
                vol_runner = context.introspect().set_sink_mute_by_index(index, true, None);
            } else {
                vol_runner = context.introspect().set_sink_volume_by_index(index, &volume, None);
            }             
            while vol_runner.get_state() == libpulse_binding::operation::State::Running {
                mainloop.iterate(false);
                success = Some(true);
            }
        } else {
            success = Some(false);
        }

        mainloop.quit(libpulse_binding::def::Retval(0));

    }

    success.unwrap_or(false)
}

pub fn set_mute(mute: bool) -> bool {
    let mut status = false;
    #[cfg(target_os="macos")] {
         #[cfg(target_os="macos")] {
        if let Ok(_) = coreaudio::set_mute(mute) {
            status = true
        } else {
            status = false;
        }
    }
    }
    #[cfg(target_os="windows")]
    {
        use windows::Win32::System::Com::CLSCTX_ALL;
        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
        use std::ptr;

        let device = get_default_output_device();
        unsafe {
            let volume_controls = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).unwrap();
            match volume_controls.SetMute(mute, ptr::null()) {
                Ok(_) => {
                    status = true;
                }
                Err(e) => {
                    debug_eprintln(&format!("Error setting mute status {}", e));
                }
            }
        
        }
    }
    #[cfg(target_os="linux")] {
        use std::sync::{Arc, Mutex};
        
        let dev_index = Arc::new(Mutex::new(None));
        let clone = Arc::clone(&dev_index);

        let mut mainloop = Mainloop::new().expect("Failed to create mainloop");
        let proplist = Proplist::new().unwrap();
        let mut context = Context::new_with_proplist(&mainloop, "CPVC", &proplist)
            .expect("Failed to create connection context");
        
        context.connect(None, libpulse_binding::context::FlagSet::NOFLAGS, None)
            .expect("Failed to connect context");

        loop {
            match context.get_state() {
                libpulse_binding::context::State::Ready => break,
                libpulse_binding::context::State::Failed | libpulse_binding::context::State::Terminated => {
                    panic!("Context failed or terminated");
                }
                _ => {
                    mainloop.iterate(false);
                }
            }
        }
        let op = context.introspect().get_sink_info_list( move |info | {
                match info {
                    libpulse_binding::callbacks::ListResult::Item(device) => {
                        if let Some(_) = device.active_port {
                            clone.lock().unwrap().replace(device.index);
                        }
                    },
                    libpulse_binding::callbacks::ListResult::End => {
                        debug_println("channel volume aquired");
                    },
                    libpulse_binding::callbacks::ListResult::Error => {
                        debug_eprintln("error gathering device information");    
                    },
                }
            });

        
        while op.get_state() == libpulse_binding::operation::State::Running {
            mainloop.iterate(false);
        }


        if let Some(index) = dev_index.lock().unwrap().take() {
            let mute_runner;
            mute_runner = context.introspect().set_sink_mute_by_index(index, mute, None);        
            while mute_runner.get_state() == libpulse_binding::operation::State::Running {
                mainloop.iterate(false);
                status = true;
            }
        } else {
            status = false;
        }
        mainloop.quit(libpulse_binding::def::Retval(0));
        
    }
    status
}

pub fn get_mute() -> bool {
    let mut mute = 0;
    #[cfg(target_os="macos")] {
        mute = match coreaudio::get_mute().unwrap() {
            true => {
                1
            }
            false => {
                0
            }
        };
    }
    #[cfg(target_os="windows")] {
        use windows::Win32::System::Com::CLSCTX_ALL;
        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
        let device = get_default_output_device();
        unsafe {
            let volume_controls = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).unwrap();
            if volume_controls.GetMute().unwrap().into() {
               mute = 1;
            }
        
        }
    }
    #[cfg(target_os="linux")] {
        use std::sync::{Arc, Mutex};

        let mute_status = Arc::new(Mutex::new(0));
        let clone = Arc::clone(&mute_status);

        let mut mainloop = Mainloop::new().expect("Failed to create mainloop");
        let proplist = Proplist::new().unwrap();
        let mut context = Context::new_with_proplist(&mainloop, "CPVC", &proplist)
            .expect("Failed to create connection context");
        
        context.connect(None, libpulse_binding::context::FlagSet::NOFLAGS, None)
            .expect("Failed to connect context");

        loop {
            match context.get_state() {
                libpulse_binding::context::State::Ready => break,
                libpulse_binding::context::State::Failed | libpulse_binding::context::State::Terminated => {
                    panic!("Context failed or terminated");
                }
                _ => {
                    mainloop.iterate(false);
                }
            }
        }
        let op = context.introspect().get_sink_info_list( move |info | {
                match info {
                    libpulse_binding::callbacks::ListResult::Item(device) => {
                        if device.mute {
                            *clone.lock().unwrap() = 1;
                        }
                    },
                    libpulse_binding::callbacks::ListResult::End => {
                        debug_println("Devices finished")
                    },
                    libpulse_binding::callbacks::ListResult::Error => {
                        debug_eprintln("error gathering device information");
                    },
                }
            });
        
        while op.get_state() == libpulse_binding::operation::State::Running {
            mainloop.iterate(false);
        }

        mainloop.quit(libpulse_binding::def::Retval(0));

        mute = *mute_status.lock().unwrap();
    }
    match mute {
        1 => {
            true
        }
        _ => {
            false
        }
    }
}

#[cfg(target_os="windows")]
fn get_default_output_device() -> windows::Win32::Media::Audio::IMMDevice {
    use windows::Win32::Media::Audio::{eRender, eMultimedia};
    use windows::Win32::Media::Audio::IMMDevice;

    unsafe {
        let enumerator = get_enumerator();
        let default_device: IMMDevice = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia).unwrap();
        // println!("Device ID {:?}", default_device.GetId().unwrap());
        default_device
    }
}

#[cfg(target_os="windows")]
unsafe fn get_enumerator() -> windows::Win32::Media::Audio::IMMDeviceEnumerator {
    use windows::core::{Error};
    use windows::Win32::Media::Audio::IMMDeviceEnumerator;
    use windows::Win32::Media::Audio::{MMDeviceEnumerator};
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED).unwrap();
        let hresult: Result<IMMDeviceEnumerator, Error> = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL);
        match hresult {
            Ok(devices) => {
                devices
            },
            Err(error) => {
                panic!("{}", error);
            }
        }
    }
}

pub fn get_default_output_dev() -> String {
    let mut device_name = String::new();
    #[cfg(target_os = "linux")] 
    {
        use std::sync::{Arc, Mutex};

        let default_dev = Arc::new(Mutex::new(String::new()));
        let clone = Arc::clone(&default_dev);

        let mut mainloop = Mainloop::new().expect("Failed to create mainloop");
        let proplist = Proplist::new().unwrap();
        let mut context = Context::new_with_proplist(&mainloop, "CPVC", &proplist)
            .expect("Failed to create connection context");
        
        context.connect(None, libpulse_binding::context::FlagSet::NOFLAGS, None)
            .expect("Failed to connect context");

        loop {
            match context.get_state() {
                libpulse_binding::context::State::Ready => break,
                libpulse_binding::context::State::Failed | libpulse_binding::context::State::Terminated => {
                    panic!("Context failed or terminated");
                }
                _ => {
                    mainloop.iterate(false);
                }
            }
        }
        let op = context.introspect().get_sink_info_list( move |info | {
                match info {
                    libpulse_binding::callbacks::ListResult::Item(device) => {
                        if let Some(_) = device.active_port {
                            *clone.lock().unwrap() = device.description.as_ref().unwrap().to_string();
                        }
                    },
                    libpulse_binding::callbacks::ListResult::End => {
                        debug_println("Devices finished")
                    },
                    libpulse_binding::callbacks::ListResult::Error => {
                        debug_eprintln("error gathering device information");
                    },
                }
            });
        
        while op.get_state() == libpulse_binding::operation::State::Running {
            mainloop.iterate(false);
        }

        mainloop.quit(libpulse_binding::def::Retval(0));

        device_name = default_dev.lock().unwrap().clone();
    }
    device_name
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
        dbg!(get_sound_devices());
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


    #[cfg(target_os="macos")]
    #[test]
    #[ignore]
    fn get_device_details() {
        println!("{}", get_default_output_dev());
        assert!(false);
    }

    #[cfg(target_os="linux")]
    #[test]
    fn get_pulse_output_devices() {
        println!("{}", get_default_output_dev());
        assert!(false);
    }
}
