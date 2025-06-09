#[cfg(target_os="macos")]
use std::process::Command;

#[cfg(target_os="linux")]
// use alsa::{card, ctl, pcm, mixer::{SelemId, Mixer, SelemChannelId}};
use alsa::mixer::{SelemId, Mixer, SelemChannelId};

mod command;
// use std::ffi::c_int;

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

    }
    #[cfg(target_os="linux")] {
        // ALSA cannot detect cards that show up in PipeWire 
        // This function records the HW audio devices that are directly connected to the device
        // Bluetooth devices are handled by PipeWire so they will not appear
        let control = ctl::Ctl::new("pipewire", false);
        let mut name = String::from("");
        match control {
            Ok(_) => {
                name = String::from("pipewire");
            },
            Err(_) => {
                let control = Mixer::new("pulse", false);
                match control {
                    Ok(_) => {
                        name = String::from("pulse");
                    },
                    Err(_) => {
                        eprintln!("CPVC only supports PipeWire and PulseAudio at the moment, please check back to see if your framework is supported");
                    }
                }
               
            }

        }
        let control = ctl::Ctl::new(&name, false).unwrap();
        let info = control.card_info().unwrap();
        dbg!(info.get_mixername().unwrap());

        use alsa::{ctl, pcm};
        let mut count = 0;
        let mut audio_devices = Vec::new();
        while let Ok(cdev) = ctl::Ctl::new(&format!("hw:{}", count), false) {
            audio_devices.push(cdev);
            count += 1;
        } 

        let output = pcm::Info::new().unwrap();
        dbg!(output.get_subdevices_count());
        // dbg!(audio_devices.into_iter().map(|ctl| ctl.card_info().unwrap().get_mixername().unwrap().to_owned()).collect::<Vec<String>>());
        devices.append(&mut audio_devices.into_iter().map(|ctl| ctl.card_info().unwrap().get_mixername().unwrap().to_owned()).collect::<Vec<String>>());
        // let mut count = 0;
        // let cdevices: Vec<c_int> = ctl::DeviceIter::new(&control).map(|x| {count += 1; x}).collect();
        // dbg!(cdevices);
        // dbg!(count);
        // let device_list:Vec<card::Card> = card::Iter::new().map(|out| out.unwrap()).collect();
        // for device in device_list {
        //     dbg!(device.get_index());
        //     devices.push(device.get_name().unwrap());
        // }
        // dbg!(devices.clone());
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
            // mixer
        }
    }
    vol
    
}



pub fn set_system_volume(percent: u8) -> bool {
    // println!("Setting vol to {}", format!("set Volume {}", (percent as f32 / 14.29 * 100.0).round() / 100.0));
    #[allow(unused_assignments)]
    let mut success = true;
    #[cfg(target_os="macos")]{
        let factor = 14.29;
        let output = Command::new("osascript").arg("-e").arg(format!("set Volume {}",(percent as f32 / factor * 100.0).round() / 100.0)).output().expect("Are you running on MacOS?");
        // dbg!(output);
        success = output.status.success();
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

pub fn get_os() -> String {
    println!("{}", env::consts::OS);
    env::consts::OS.to_string()
}


#[cfg(test)]
mod tests {
    use std::env;
    use super::*;

    #[test]
    // #[ignore]
    fn test_os() {
        println!("{}", get_os());
        assert_eq!(env::consts::OS, get_os());
    }

    #[test] 
    // #[ignore]
    fn sound_devices() {
        dbg!("{}", get_sound_devices());
        assert_eq!(false, true);
    }

    #[test]
    fn current_output() {
        dbg!(set_system_volume(24));
        assert!(set_system_volume(24));
    }

     #[test] 
    // #[ignore]
    fn sound_devices_cmd() {
        dbg!("{}", command::get_sound_devices_command());
        assert_eq!(false, true);
    }

    #[test]
    fn current_output_cmd() {
        dbg!(command::set_system_volume_command(24));
        assert!(command::set_system_volume_command(24));
    }
}