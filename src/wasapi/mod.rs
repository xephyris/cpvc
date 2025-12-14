use crate::{debug_eprintln, device::Error};

pub trait VolumeControl {
    fn get_sound_devices() -> Result<Vec<String>, Error>;

    fn get_vol() -> Result<f32, Error>;

    fn set_vol(value: f32) -> Result<(), Error>;

    fn get_mute() -> Result<bool, Error>;

    fn set_mute(state: bool) -> Result<(), Error>;
}

pub fn get_sound_devices() -> Result<Vec<String>, Error> {
    use windows::Win32::Media::Audio::{eRender, DEVICE_STATE_ACTIVE};
    use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
    use windows::Win32::System::Com::STGM_READ;
    let mut devices: Vec<String> = Vec::new();
    unsafe {
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
    Ok(devices)
}


pub fn get_vol() -> Result<f32, Error> {
    use windows::Win32::System::Com::CLSCTX_ALL;
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    let mut vol: f32 = 0.0;
    let device = get_default_output_device();
    unsafe {
        let volume_controls = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).unwrap();
        if volume_controls.GetMute().unwrap().into() {
            vol = 0.0;
        } else {
            let channel_count = volume_controls.GetChannelCount().unwrap();
            let mut total_volumes = 0.0;
            for channel in 0..channel_count {
                total_volumes += volume_controls.GetChannelVolumeLevelScalar(channel).unwrap();
            }
            total_volumes *= 100.0;
            vol = (total_volumes / channel_count as f32);
        }

        // dbg!(volume_controls);
    }

    Ok(vol)
}

pub fn set_vol(value: f32) -> Result<(), Error> {
    use windows::Win32::System::Com::CLSCTX_ALL;
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use std::ptr;

    let mut success = None;

    let device = get_default_output_device();
    unsafe {
        let volume_controls = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).unwrap();
        if volume_controls.GetMute().unwrap().into() {
            volume_controls.SetMute(false, ptr::null()).unwrap();
        }

        let channel_count = volume_controls.GetChannelCount().unwrap();
        for channel in 0..channel_count {
            volume_controls.SetChannelVolumeLevelScalar(channel, value, ptr::null()).unwrap();
        }

        if value == 0.0 {
            volume_controls.SetMute(true, ptr::null()).unwrap();
        }
    }
    
    success.replace(true);
    match success {
        Some(_) => {
            Ok(())
        },
        None => {
            Err(Error::DeviceCaptureFailed)
        }
    }
}

pub fn get_mute() -> Result<bool, Error> {
    use windows::Win32::System::Com::CLSCTX_ALL;
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    let mut mute = 0;
    let device = get_default_output_device();
    unsafe {
        let volume_controls = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).unwrap();
        if volume_controls.GetMute().unwrap().into() {
            mute = 1;
        }
    
    }

    match mute {
        1 => {
            Ok(true)
        }
        _ => {
            Ok(false)
        }
    }
}

pub fn set_mute(mute: bool) -> Result<(), Error> {
    use windows::Win32::System::Com::CLSCTX_ALL;
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use std::ptr;
    let mut status = false;
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

    match status {
        true => {
            Ok(())
        },
        false => {
            Err(Error::DeviceCaptureFailed)
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
pub unsafe fn get_enumerator() -> windows::Win32::Media::Audio::IMMDeviceEnumerator {
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