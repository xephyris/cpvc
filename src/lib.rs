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

use std::env;

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

#[derive(Debug, Clone, Copy, PartialEq)]
enum DeviceType {
    Input,
    Output,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Error {
    OutputDeviceCaptureError,
    DeviceDetailsCaptureError,
    NameCaptureError,
}



/// Gathers the human readable device name of each output device detected
pub fn get_sound_devices() -> Vec<String> {
    let mut devices:Vec<String> = Vec::new();
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
                    let name = get_device_name(*device).unwrap();
                    match check_device_type(*device) {
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
        }
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
        let captured_device_id = capture_output_device_id();
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
                let device_details = get_output_device_details(device_id);
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
                    }
                }
            } else {
                vol = 0;
            }
        }
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
    #[cfg(target_os="macos")]{

        let captured_device_id = capture_output_device_id();
        if captured_device_id.is_ok() {
            let device_id = captured_device_id.unwrap();
            let device_details = get_output_device_details(device_id);

            if device_details.is_ok() {
                let channel_count = device_details.unwrap().mChannelsPerFrame;

                let volume = percent as f32 / 100 as f32;
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
                if percent == 0 {
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
                if success.is_none() {
                    success.replace(sync_status);
                }
            } else {
                success.replace(false);
            }
        }
        success.unwrap_or(false);
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
    #[cfg(target_os = "macos")]
    {
        let captured_device_id = capture_output_device_id();
        if captured_device_id.is_ok() {
            let name = get_device_name(captured_device_id.unwrap());
            if name.is_ok() {
                device_name.push_str(&name.unwrap());
            }
        }
    }
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
fn check_device_type(device_id: u32) -> DeviceType {
    let dev_type_address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyStreams,
        mScope: kAudioObjectPropertyScopeOutput,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut stream_count: u32 = 0;
    let count_size = size_of::<u32>() as u32;
    let capture_type_status;
    unsafe {
        capture_type_status = AudioObjectGetPropertyData(
            device_id,
            NonNull::new_unchecked(&dev_type_address as *const _ as *mut _),
            0,
            null(),
            NonNull::new_unchecked(&count_size as *const _ as *mut _),
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

#[cfg(target_os="macos")]
fn get_device_name(device_id: u32) -> Result<String, Error> {
    #[cfg(target_os = "macos")]
    {
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
                Ok(CFString::wrap_under_get_rule(name).to_string())
            } else {
                debug_eprintln(&format!("Failed to get device name. Status: {}", status));
                Err(Error::NameCaptureError)
            }
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
        dbg!(set_system_volume(0));
        dbg!(get_system_volume());
        assert!(false);
    }


    // #[test]
    // #[ignore]
    // fn current_output() {
    //     dbg!(set_system_volume(0));
    //     assert!(false);
    // }

    // #[test]
    // #[ignore]
    // fn sound_devices_cmd() {a
    //     assert_eq!(false, true);
    // }

    // #[test]
    // #[ignore]
    // fn current_output_cmd() {
    //     assert!(command::set_system_volume_command(24));
    // }

    #[cfg(target_os="macos")]
    #[test]
    #[ignore]
    fn get_device_details() {
        println!("{}", get_default_output_dev());
        get_output_device_details(capture_output_device_id().unwrap()).unwrap();
        assert!(false);
    }

    #[cfg(target_os="linux")]
    #[test]
    fn get_pulse_output_devices() {
        println!("{}", get_default_output_dev());
        assert!(false);
    }
}
