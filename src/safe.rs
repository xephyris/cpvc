#[cfg(target_os = "macos")]
use crate::coreaudio;
#[cfg(target_os = "linux")]
use crate::pulseaudio;
#[cfg(target_os = "windows")]
use crate::wasapi;

pub type Result<T> = std::result::Result<T, super::error::Error>;

/// Gathers the human readable device name of each output device detected
pub fn get_sound_devices() -> Result<Vec<String>> {
    #[cfg(target_os = "macos")]
    {
        return coreaudio::get_sound_devices();
    }
    #[cfg(target_os = "windows")]
    {
        return wasapi::get_sound_devices();
    }
    #[cfg(target_os = "linux")]
    {
        return pulseaudio::PulseAudio::get_sound_devices();
    }
}

/// Gathers the current volume in percent of the default output device
pub fn get_system_volume() -> Result<u8> {
    #[cfg(target_os = "macos")]
    {
        return Ok((coreaudio::get_vol()? * 100.0) as u8);
    }
    #[cfg(target_os = "windows")]
    {
        // println!("{}", wasapi::WASAPI::get_vol()?);
        return Ok((wasapi::get_vol()? * 100.0) as u8);
    }
    #[cfg(target_os = "linux")]
    {
        return Ok((pulseaudio::PulseAudio::get_vol()? * 100.0) as u8);
    }
}

/// Sets the current volume in percent of the default output device
/// ## On macOS
/// `cpvc` needs to mute and unmute the audio device to get the hardware device volume to sync
pub fn set_system_volume(percent: u8) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        return coreaudio::set_vol(percent as f32 / 100.0);
    }
    #[cfg(target_os = "windows")]
    {
        return wasapi::set_vol(percent as f32 / 100.0);
    }
    #[cfg(target_os = "linux")]
    {
        return pulseaudio::PulseAudio::set_vol(percent as f32 / 100.0);
    }
}

pub fn set_mute(mute: bool) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        return coreaudio::set_mute(mute);
    }
    #[cfg(target_os = "windows")]
    {
        return wasapi::set_mute(mute);
    }
    #[cfg(target_os = "linux")]
    {
        return pulseaudio::PulseAudio::set_mute(mute);
    }
}

pub fn get_mute() -> Result<bool> {
    #[cfg(target_os = "macos")]
    {
        return coreaudio::get_mute();
    }
    #[cfg(target_os = "windows")]
    {
        return wasapi::get_mute();
    }
    #[cfg(target_os = "linux")]
    {
        return pulseaudio::PulseAudio::get_mute();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_devices() -> Result<()> {
        dbg!(get_sound_devices()?);
        Ok(())
    }

    #[test]
    fn set_sound_test() -> Result<()> {
        dbg!(set_system_volume(2)?);
        Ok(())
    }

    #[test]
    fn get_sound_test() -> Result<()> {
        dbg!(get_system_volume()?);
        Ok(())
    }

    #[test]
    fn set_mute_test() -> Result<()> {
        dbg!(set_mute(true)?);
        dbg!(get_system_volume()?);
        Ok(())
    }

    #[test]
    fn get_mute_status() -> Result<()> {
        dbg!(get_mute()?);
        dbg!(get_system_volume()?);
        Ok(())
    }
}
