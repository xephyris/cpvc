use windows::core::PWSTR;
use windows::Win32::System::Com::CLSCTX_ALL;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{DEVICE_STATE_ACTIVE, IMMDevice, eMultimedia, eRender};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::System::Com::STGM_READ;
use crate::VolumeControl;
use crate::wasapi::device::WASAPIDevice;
use crate::{debug_eprintln, error::Error};

pub mod device;



pub struct WASAPI {}

impl VolumeControl for WASAPI {
    fn get_sound_devices() -> Result<Vec<String>, Error> {
         Ok(WASAPI::get_device_identifiers()?.into_iter().map(|(_pwstr, name)| name).collect())
    }

    fn get_vol() -> Result<f32, Error> {
        let default_device = WASAPI::get_default_output_device()?;
        Ok(default_device.get_vol()?)
    }

    fn set_vol(value: f32) -> Result<(), Error> {
        let default_device = WASAPI::get_default_output_device()?;
        Ok(default_device.set_vol(value)?)
    }

    fn get_mute() -> Result<bool, Error> {
        let default_device = WASAPI::get_default_output_device()?;
        Ok(default_device.get_mute()?)
    }

    fn set_mute(state: bool) -> Result<(), Error> {
        let default_device = WASAPI::get_default_output_device()?;
        Ok(default_device.set_mute(state)?)
    }
}

impl WASAPI {
    
    pub fn get_device_identifiers() -> Result<Vec<(String, String)>, Error> {
        let mut devices: Vec<(String, String)> = Vec::new();
        unsafe {
            let enumerator = WASAPI::get_enumerator();
            let device_col = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE).unwrap();
            let dev_count = device_col.GetCount().unwrap();
            for device_id in 0..dev_count{
                let device = device_col.Item(device_id).unwrap();
                let result = device.OpenPropertyStore(STGM_READ);
                match result {
                    Ok(properties) => {
                        let name = properties.GetValue(&PKEY_Device_FriendlyName).unwrap();
                        let uid = WASAPI::get_imm_device_uid(&device)?;
                        devices.push((uid, name.to_string()));
                        // dbg!(properties.GetValue(&PKEY_Device_FriendlyName));
                    },
                    Err(error) => {
                        return Err(Error::DeviceEnumerationFailed(format!("Failed to get audio devices {error}")))
                    }
                }
            }
        }
        Ok(devices)
    }

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

    pub fn get_default_output_device() -> Result<WASAPIDevice, Error> {
        let default_device;
        unsafe {
            let enumerator = WASAPI::get_enumerator();
            default_device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia).unwrap();
        }
        Ok(WASAPIDevice::from_imm_device(default_device)?)
    }

    pub fn get_imm_device_uid(device: &IMMDevice) -> Result<String, Error> {
        unsafe {
            Ok(device.GetId().map_err(|e| Error::DeviceAccessFailed(format!("Failed to get Device Id {e}")))?
                .to_string().map_err(|e| Error::DeviceAccessFailed(format!("PWSTR conversion failed {e}")))?)
        }
    }
}