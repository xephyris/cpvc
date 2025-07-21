#[cfg(target_os="macos")]
use std::process::Command;

#[cfg(target_os="linux")]
// use alsa::{card, ctl, pcm, mixer::{SelemId, Mixer, SelemChannelId}};
use alsa::{ctl, mixer::{SelemId, Mixer, SelemChannelId}};

pub mod command;

use std::env;

pub fn get_sound_devices() -> Vec<String> {
    let mut devices:Vec<String> = Vec::new();
    #[cfg(target_os="macos")] {
        let output = Command::new("system_profiler").arg("SPAudioDataType").output().expect("Are you running on MacOS?");
        let lines:Vec<String> = String::from_utf8_lossy(&output.stdout).to_string().lines().map(|str| str.to_owned()).collect();
        for (num, line) in lines.iter().enumerate() {
            if !line.contains("          ") && num > 3 && line != ""{
                if !lines.get(num + 3).unwrap().contains("Input") {
                    devices.push(line.trim().replace(":", "").to_owned());
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
    let mut vol: u8 = 0;
    #[cfg(target_os="macos")] {
        let output = Command::new("osascript").arg("-e").arg("return output volume of (get volume settings)").output().expect("Are you running on MacOS?");
        let out = String::from_utf8_lossy(&output.stdout).to_string().trim().to_owned();
        vol = out.parse::<u8>().unwrap_or(0);
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
    let mut success = None;
    #[cfg(target_os="macos")]{
        let factor = 14.29;
        let output = Command::new("osascript").arg("-e").arg(format!("set Volume {}",(percent as f32 / factor * 100.0).round() / 100.0)).output().expect("Are you running on MacOS?");
        success.replace(output.status.success());
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
        assert_eq!(false, true);
    }

    #[test]
    fn current_output() {
        dbg!(set_system_volume(0));
        
        assert!(false);
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
}