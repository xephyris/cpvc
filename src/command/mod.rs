use std::process::Command;

#[allow(dead_code)]
pub fn get_system_volume_command() -> u8 {
    #[allow(unused_assignments)]
    let mut vol = 0;
    #[cfg(target_os="macos")] {
        let output = Command::new("osascript").arg("-e").arg("return output volume of (get volume settings)").output().expect("Are you running on MacOS?");
        let out = String::from_utf8_lossy(&output.stdout).to_string().trim().to_owned();
        vol = out.parse::<u8>().unwrap_or(0);
    }
    #[cfg(target_os="linux")] {
        use std::process::Stdio;
        let mixer = Command::new("amixer").arg("sget").arg("Master").stdout(Stdio::piped()).spawn().unwrap();
        let channels = Command::new("grep").arg("-o").arg("[0-9]*%").stdin(mixer.stdout.unwrap()).stdout(Stdio::piped()).spawn().unwrap();
        let channel_vols = Command::new("tr").arg("-d").arg("%").stdin(channels.stdout.unwrap()).output().unwrap();
        let volumes:String = channel_vols.stdout.into_iter().map(|chr| chr as char).collect();
        let volumes:Vec<u8> = volumes.trim().split("\n").map(|num| num.parse().unwrap_or(0)).collect();
        vol = (volumes.iter().map(|x| *x as f32).sum::<f32>() / volumes.len() as f32) as u8;
    }
    vol
    
}


#[allow(dead_code)]
pub fn set_system_volume_command(percent: u8) -> bool {
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
        let command = Command::new("amixer").arg("-D").arg("pipewire").arg("sset").arg("Master").arg(format!("{}%", percent)).output().unwrap();
        dbg!(command.clone());
        if command.stderr.len() > 0 {
            let retry = Command::new("amixer").arg("-D").arg("pulse").arg("sset").arg("Master").arg(format!("{}%", percent)).output().unwrap();
            if retry.stderr.len() > 0 {
                success = false;
            }
        }
    }
    success
}

#[allow(dead_code)]
pub fn get_sound_devices_command() -> Vec<String> {
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
        let output = Command::new("pw-cli").arg("ls").arg("Node").output().unwrap();
        if output.stderr.len() == 0 {
            let mut contents:String = output.stdout.into_iter().map(|chr| chr as char).collect();
            contents = contents.replace("\t\t", "");
            let contents:Vec<String> = contents.split("\t").map(|x| x.to_owned()).collect();
            let contents:String = contents.into_iter().filter(|item| item.contains("media.class = \"Audio/Sink\"")).collect();
            devices = contents.split("\n").map(|i| i.to_owned()).filter(|x| x.contains("node.description")).map(|dev| dev.replace(" node.description = ", "").replace("\"", "")).collect();
            dbg!(devices.clone());
        }
    }
    devices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    // #[ignore]
    fn sound_devices() {
        dbg!("{}", get_sound_devices_command());
        assert_eq!(false, true);
    }

    #[test]
    fn current_output() {
        dbg!(set_system_volume_command(24));
        assert!(set_system_volume_command(24));
    }
}