use windows::{Win32::{Devices::FunctionDiscovery::PKEY_Device_FriendlyName, Media::Audio::{Endpoints::IAudioEndpointVolume, IMMDevice}, System::Com::{CLSCTX_ALL, STGM_READ}}, core::PWSTR,};
use std::ptr;
use crate::{debug_eprintln, error::Error, wasapi::WASAPI};

pub struct WASAPIDevice {
    device: IMMDevice
}

impl WASAPIDevice {
    // PWSTRs are not constant and keep changing even between lines, do not use for matching

    pub fn from_name(name: String) -> Result<Self, Error> {
        let devices = WASAPI::get_device_identifiers()?;
        let mut uid = None;
        
        for (u_id, names) in devices {
            if name == names {
                uid = Some(u_id);   
                break
            }
        }
        if let Some(uid) = uid {
            let mut id = format!("{}\0", uid).encode_utf16().collect::<Vec<u16>>(); 
            let pwstr = PWSTR(id.as_mut_ptr());
            let device;
            unsafe {
                let enumerator = WASAPI::get_enumerator();
                device = enumerator.GetDevice(pwstr).map_err(|e| Error::DeviceAccessFailed(format!("Failed to capture IMMDevice {e}")))?;
            }
            Ok(Self {
                device,
            })
        } else {
            Err(Error::DeviceNotFound)
        }
    }

    pub fn from_uid(uid: String) -> Result<Self, Error> {
        let devices = WASAPI::get_device_identifiers()?;
        let mut matched = false;
        for (u_id, _name) in devices {
            if uid == u_id {
                matched = true;
                break
            }
        }
        let mut id = format!("{}\0", uid).encode_utf16().collect::<Vec<u16>>(); 
        let pwstr = PWSTR(id.as_mut_ptr());
        if matched {
            let device;
            unsafe {
                let enumerator = WASAPI::get_enumerator();
                device = enumerator.GetDevice(pwstr).map_err(|e| Error::DeviceAccessFailed(format!("Failed to capture IMMDevice {e}")))?;
            }
            Ok(Self {
                device
            })
        } else {
            Err(Error::DeviceNotFound)
        }
    }

    pub fn from_imm_device(device: IMMDevice) -> Result<Self, Error> {
        let uid; 
        uid = WASAPI::get_imm_device_uid(&device)?;

        let devices = WASAPI::get_device_identifiers()?;
        let mut matched = false;
        for (u_id, _name) in devices {
            if uid == u_id {
                matched = true;
                break
            }
        }
        if matched {
            Ok(Self {
                device
            })
        } else {
            Err(Error::DeviceNotFound)
        }
    }

    pub fn get_device_uid(&self) -> Result<String, Error> {
        unsafe {
            Ok(self.device.GetId().map_err(|e| Error::DeviceAccessFailed(format!("Failed to get Device Id {e}")))?
                .to_string().map_err(|e| Error::DeviceAccessFailed(format!("PWSTR conversion failed {e}")))?)
        }
    }

    pub fn get_name(&self) -> Result<String, Error> {
        let result = unsafe {self.device.OpenPropertyStore(STGM_READ)};
        match result {
            Ok(properties) => {
                return Ok(unsafe {properties.GetValue(&PKEY_Device_FriendlyName).map_err(|e| Error::DeviceAccessFailed(format!("Failed to access property store values {e}")))}?.to_string());
            },
            Err(error) => {
                return Err(Error::DeviceAccessFailed(format!("Failed to access property store {error}")))
            }
        }
    }

    pub fn get_vol(&self) -> Result<f32, Error> {
        let mut vol: f32 = 0.0;
        unsafe {
            let volume_controls = self.device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).unwrap();
            if volume_controls.GetMute().unwrap().into() {
                vol = 0.0;
            } else {
                let channel_count = volume_controls.GetChannelCount().unwrap();
                let mut total_volumes = 0.0;
                for channel in 0..channel_count {
                    total_volumes += volume_controls.GetChannelVolumeLevelScalar(channel).unwrap();
                }
                vol = (total_volumes / channel_count as f32);
            }

            // dbg!(volume_controls);
        }

        Ok(vol)
    }

    pub fn set_vol(&self, value: f32) -> Result<(), Error> {
        let mut success = None;
        unsafe {
            let volume_controls = self.device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).unwrap();
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
                Err(Error::Placeholder)
            }
        }
    }

    pub fn get_mute(&self) -> Result<bool, Error> {
        let mut mute = 0;
        unsafe {
            let volume_controls = self.device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).unwrap();
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

    pub fn set_mute(&self, mute: bool) -> Result<(), Error> {
        let mut status = false;
        unsafe {
            let volume_controls = self.device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).unwrap();
            match volume_controls.SetMute(mute, ptr::null()) {
                Ok(_) => {
                    status = true;
                }
                Err(e) => {
                    return Err(Error::MuteSetFailed(format!("Error setting mute status {}", e)));
                }
            }
        
        }

        match status {
            true => {
                Ok(())
            },
            false => {
                Err(Error::Placeholder)
            }
        }
    }
}