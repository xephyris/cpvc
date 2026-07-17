use std::process::Command;

#[allow(dead_code)]
pub fn get_system_volume_command(device_uid: String) -> u8 {
    #[allow(unused_assignments)]
    let mut vol = 0;
    #[cfg(target_os="macos")] {
        eprintln!("cpvc::command is primarily used for testing verification. Do not use for production!");
        let output = Command::new("osascript").arg("-e").arg("return output volume of (get volume settings)").output().expect("Are you running on MacOS?");
        let out = String::from_utf8_lossy(&output.stdout).to_string().trim().to_owned();
        vol = out.parse::<u8>().unwrap_or(0);
    }
    #[cfg(target_os="windows")]{
        eprintln!("cpvc::command is primarily used for testing verification. Do not use for production!");
        let output = 
            if !device_uid.is_empty() {
                Command::new("powershell.exe").args(["./ext_tests/wasapi_tester.ps1", "-GetVolume", "-Id", &device_uid]).output()
            } else {
                Command::new("powershell.exe").args(["./ext_tests/wasapi_tester.ps1", "-GetVolume"]).output()
            };
        if let Ok(out) = output {
            let volume_str = String::from_utf8_lossy(&out.stdout);
            let volume_str = volume_str.trim();
            dbg!(volume_str);
            let volume = volume_str.parse::<u8>().unwrap();
            let vol_f32 = volume as f32 / 100.0;
            dbg!(vol_f32);
        }
        
    }
    #[cfg(target_os="linux")] {
        eprintln!("cpvc::command is primarily used for testing verification. Do not use for production!");
        let output = 
            if !device_uid.is_empty() {
                Command::new("bash").args(["./ext_tests/pulseaudio_tester.sh", "--get-vol", &device_uid]).output()
            } else {
                Command::new("bash").args(["./ext_tests/pulseaudio_tester.sh", "--get-vol"]).output()
            };
        if let Ok(out) = output {
            let volume_str = String::from_utf8_lossy(&out.stdout);
            let volume_str = volume_str.trim();
            dbg!(volume_str);
            let volume = volume_str.parse::<f32>().unwrap();
            let vol_f32 = volume as f32 / 100.0;
            dbg!(vol_f32);
        }
    }
    vol
    
}


#[allow(dead_code)]
pub fn set_system_volume_command(device_uid: String, percent: u8) -> bool {
    
    #[allow(unused_assignments)]
    let mut success = true;
    #[cfg(target_os="macos")]{
        eprintln!("cpvc::command is primarily used for testing verification. Do not use for production!");
        let factor = 14.29;
        let output = Command::new("osascript").arg("-e").arg(format!("set Volume {}",(percent as f32 / factor * 100.0).round() / 100.0)).output().expect("Are you running on MacOS?");
        success = output.status.success();
    }
    #[cfg(target_os="windows")]{
        eprintln!("cpvc::command is primarily used for testing verification. Do not use for production!");
    }
    #[cfg(target_os="linux")] {
        eprintln!("cpvc::command is primarily used for testing verification. Do not use for production!");
        let output = 
            if !device_uid.is_empty() {
                Command::new("bash").args(["./ext_tests/pulseaudio_tester.sh", "--set-vol", &device_uid, &(percent as f32 / 100.0).to_string()]).output()
            } else {
                Command::new("bash").args(["./ext_tests/pulseaudio_tester.sh", "--set-vol", "", &(percent as f32 / 100.0).to_string()]).output()
            };
        if let Ok(out) = output {
            let volume_str = String::from_utf8_lossy(&out.stdout);
            let volume_str = volume_str.trim();
            dbg!(volume_str);
            let volume = volume_str.parse::<f32>().unwrap();
            let vol_f32 = volume as f32 / 100.0;
            dbg!(vol_f32);
        }
    }
    success
}

#[allow(dead_code)]
pub fn get_sound_devices_command() -> Vec<String> {
    let mut devices:Vec<String> = Vec::new();
    #[cfg(target_os="macos")] {
        eprintln!("cpvc::command is primarily used for testing verification. Do not use for production!");
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
        eprintln!("cpvc::command is primarily used for testing verification. Do not use for production!");
    }
    #[cfg(target_os="linux")] {
        eprintln!("cpvc::command is primarily used for testing verification. Do not use for production!");
        let output = Command::new("pw-cli").arg("ls").arg("Node").output().unwrap();
        if output.stderr.len() == 0 {
            let mut contents:String = output.stdout.into_iter().map(|chr| chr as char).collect();
            contents = contents.replace("\t\t", "");
            let contents:Vec<String> = contents.split("\t").map(|x| x.to_owned()).collect();
            let contents:String = contents.into_iter().filter(|item| item.contains("media.class = \"Audio/Sink\"")).collect();
            devices = contents.split("\n").map(|i| i.to_owned()).filter(|x| x.contains("node.description")).map(|dev| dev.replace(" node.description = ", "").replace("\"", "")).collect();
        }
    }
    devices
}

#[cfg(test)]
mod tests {
    use crate::*;
    use super::*;

    #[test]
    fn verify_command_output() {
        get_system_volume_command("".to_string());
        set_system_volume_command("".to_string(), 10);
        assert!(false);
    }

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

}
