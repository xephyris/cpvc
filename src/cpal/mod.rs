// use std::ffi::c_void;

// #[cfg(target_os = "macos")]
// use core_foundation::{base::TCFType, string::CFString};
// pub use cpal::*;

// #[cfg(target_os="macos")]
// use objc2_core_audio::{
//         AudioObjectGetPropertyData, AudioObjectSetPropertyData, AudioObjectGetPropertyDataSize,
//         AudioObjectID, AudioObjectPropertyAddress,
//         kAudioHardwarePropertyDefaultOutputDevice, kAudioObjectSystemObject,
//         kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyElementMain,
//         kAudioDevicePropertyScopeOutput, kAudioDevicePropertyMute,
//         kAudioDevicePropertyVolumeScalar, kAudioDevicePropertyDeviceNameCFString,
//         kAudioDevicePropertyStreamFormat, kAudioObjectPropertyScopeOutput,
//         kAudioHardwarePropertyDevices, kAudioDevicePropertyStreams,
//         kAudioObjectPropertyScopeInput, kAudioHardwarePropertyDeviceForUID
// };
// use crate::{debug_eprintln, error::Error};

// pub trait VolumeControlExt {
//     fn default_volume_control(&self) -> Result<VolControl, VolumeError>;
// }

// impl VolumeControlExt for cpal::Device {
//     // fn default_volume_control(&self) -> Result<VolControl, VolumeError> {
//     //      #[cfg(target_os="macos")] {
//     //         use cpal::traits::DeviceTrait;

//     //         use crate::{coreaudio::get_output_device_details, scan::scan_devices};
//     //         let name = self.name().unwrap();
//     //         if let Some(device_id) = scan_devices().remove(&name) {
//     //             let channels;
                
//     //             let device_stats = get_output_device_details(device_id);
//     //             if let Ok(stats) = device_stats {
//     //                 channels = stats.mChannelsPerFrame;
//     //             } else {
//     //                 return Err(VolumeError::ChannelCountCaptureError);
//     //             }
//     //             return Ok(VolControl::new(device_id, channels));
//     //         }
//     //         return Err(VolumeError::DeviceNotFound);        
//     //     }
//     //     Err(VolumeError::UnsupportedOS)
//     // }
//     fn default_volume_control(&self) -> Result<VolControl, VolumeError> {
//         #[cfg(target_os="macos")] {
//             use cpal::traits::DeviceTrait;

//             use crate::{coreaudio::CoreAudio};
//             let id = self.id().unwrap();
//             // if let Some(device_id) = scan_devices().remove(&name) {
//             //     let channels;
                
//             //     let device_stats = get_output_device_details(device_id);
//             //     if let Ok(stats) = device_stats {
//             //         channels = stats.mChannelsPerFrame;
//             //     } else {
//             //         return Err(VolumeError::ChannelCountCaptureError);
//             //     }
//             //     return Ok(VolControl::new(device_id, channels));
//             // }
//             // return Err(VolumeError::DeviceNotFound);        
//         }
//         Err(VolumeError::UnsupportedOS)
//     }
// }

// #[derive(Debug, Clone)]
// pub struct VolControl {
//     id: u32,
//     channels: u32,

// }

// impl VolControl {
//     pub fn new(hw_id: u32, channels: u32) -> VolControl {
//         VolControl { id:hw_id, channels }
//     }

//     // pub fn from_cpal_id(device_id: DeviceId) -> Self {
//     //     match device_id.0 {
//     //         HostId::CoreAudio => todo!(),
//     //         // HostId::Jack => {}
//     //     }
//     // }

//     pub fn set_vol(&self, val: f32) -> Result<(), VolumeError> {
//         let mut success = Some(false);
//         #[cfg(target_os="macos")]{
//             use std::ptr::{null, NonNull};
//             let channel_count = self.channels;

//             let volume_data_size = size_of::<f32>() as u32;

//             for channel in 0..=channel_count {
//                 debug_eprintln(&format!("channel {}", channel));
//                 let volume_property_address_channel = AudioObjectPropertyAddress {
//                     mSelector: kAudioDevicePropertyVolumeScalar,
//                     mScope: kAudioDevicePropertyScopeOutput,
//                     mElement: channel,
//                 };

//                 unsafe {
//                     let change_volume_status = AudioObjectSetPropertyData(self.id,
//                         NonNull::new_unchecked(&volume_property_address_channel as *const _ as *mut _),
//                         0, null(),
//                         volume_data_size, NonNull::new_unchecked( &val as *const _ as *mut _));
//                     if change_volume_status != 0 {
//                         debug_eprintln(&format!("Failed to change volume on channel {} (This may be normal behavior)", if channel == 0 {"0 (Master Channel)".to_string()} else {channel.to_string()}));
//                     }
//                 }
//             }

//             let mute_property_address = AudioObjectPropertyAddress {
//                 mSelector: kAudioDevicePropertyMute,
//                 mScope: kAudioDevicePropertyScopeOutput,
//                 mElement: kAudioObjectPropertyElementMain
//             };

//             let mut sync_status = true;
//             // Mute then unmute hardware device so software sound level will sync with hardware sound level
//             for mute in (0..=1 as u32).rev() {
//                 let mute_data_size = size_of::<u32>() as u32;
//                 unsafe {
//                     let mute_status = AudioObjectSetPropertyData(self.id,
//                         NonNull::new_unchecked(&mute_property_address as *const _ as *mut _),
//                         0, null(),
//                         mute_data_size, NonNull::new_unchecked(&mute as *const _ as *mut _));
//                     if mute_status != 0 {
//                         sync_status = false;
//                     }
//                 }
//             }
//             if sync_status == false {
//                 return Err(VolumeError::VolumeChangeError)
//             }
//         }
//         match success.take() {
//             Some(value) => {
//                 if value {
//                     return Ok(());
//                 } else {
//                     return Err(VolumeError::VolumeChangeError);
//                 }
//             },
//             None => {
//                 return Err(VolumeError::VolumeChangeError)
//             }
//         }
//     }
//     pub fn get_vol(&self) -> f32 {
//         let mut vol = 0.0;
//         #[cfg(target_os="macos")] {
//             use std::ptr::{NonNull, null}; 

//             // Check if Muted
//             let channel_count = self.channels;
//             let mut total_volume: f32 = 0.0;
//             let mut total_channels = 0;
//             let mut channel_volume: f32 = 0.0;
//             let mut volume_data_size = size_of::<f32>() as u32;

//             for channel in 0..=channel_count {
//                 let volume_property_address_channel = AudioObjectPropertyAddress {
//                     mSelector: kAudioDevicePropertyVolumeScalar,
//                     mScope: kAudioDevicePropertyScopeOutput,
//                     mElement: channel,
//                 };

//                 unsafe {


//                     let get_volume_data_size_status = AudioObjectGetPropertyDataSize(
//                             self.id,
//                             NonNull::new_unchecked(&volume_property_address_channel as *const _ as *mut _),
//                             0,
//                             null(),
//                             NonNull::new_unchecked(&mut volume_data_size as *const _ as *mut _),
//                         );
//                     if get_volume_data_size_status == 0 {
//                         let get_volume_status = AudioObjectGetPropertyData(
//                             self.id,
//                             NonNull::new_unchecked(&volume_property_address_channel as *const _ as *mut _),
//                             0,
//                             null(),
//                             NonNull::new_unchecked(&volume_data_size as *const _ as *mut _),
//                             NonNull::new_unchecked(&mut channel_volume as *mut _ as *mut _));

//                         if get_volume_status != 0 {
//                             debug_eprintln(&format!("Failed to get volume on channel {} (This may be normal behavior)", if channel == 0 {"0 (Master Channel)".to_string()} else {channel.to_string()}));
//                         } else {
//                             total_channels += 1;
//                             total_volume += channel_volume;
//                         }
//                     } else {
//                         debug_eprintln(&format!("Failed to get volume data size on channel {} (This may be normal behavior)", if channel == 0 {"0 (Master Channel)".to_string()} else {channel.to_string()}));
//                     }
//                 }
//             }
//             if total_channels > 0 {
//                 total_volume *= 100.0;
//                 total_volume = total_volume.round();
//                 vol = (total_volume as u32 / total_channels) as f32;
//             }
//         }
//         vol.into()
//     }
//     pub fn set_mute(&self, mute: bool) -> bool {
//         let mut status = false;
//         #[cfg(target_os="macos")] {
//             let mute_property_address = AudioObjectPropertyAddress {
//                         mSelector: kAudioDevicePropertyMute,
//                         mScope: kAudioDevicePropertyScopeOutput,
//                         mElement: kAudioObjectPropertyElementMain
//                     };
//             let mute_data_size = size_of::<u32>() as u32;
//             let mute = match mute {
//                 true => {
//                     1
//                 },
//                 false => {
//                     0
//                 }
//             };
//             unsafe {
//                 use std::ptr::{NonNull, null};
//                 let mute_status = AudioObjectSetPropertyData(self.id,
//                     NonNull::new_unchecked(&mute_property_address as *const _ as *mut _),
//                     0, null(),
//                     mute_data_size, NonNull::new_unchecked(&mute as *const _ as *mut _));
//                 if mute_status != 0 {
//                     status = false;
//                 }
//             }
//         }
//         status
//     }

//      pub fn is_mute(&self) -> Result<bool, VolumeError> {
//         let mut mute:u32 = 0;
//         #[cfg(target_os="macos")] {
//             let mute_property_address = AudioObjectPropertyAddress {
//                         mSelector: kAudioDevicePropertyMute,
//                         mScope: kAudioDevicePropertyScopeOutput,
//                         mElement: kAudioObjectPropertyElementMain
//                     };
//             let mut mute_data_size = size_of::<u32>() as u32;

//             unsafe {
//                 use std::ptr::{NonNull, null};
//                 let mute_status = AudioObjectGetPropertyData(self.id,
//                     NonNull::new_unchecked(&mute_property_address as *const _ as *mut _),
//                     0, null(),
//                     NonNull::new_unchecked(&mut mute_data_size as *mut _), 
//                     NonNull::new_unchecked(&mut mute as *mut _ as *mut _));
//                 if mute_status != 0 {
//                     return Err(VolumeError::MuteStatusCaptureFailed);
//                 }
//             }
//         }
//         Ok(match mute {
//             1 => {
//                 true
//             }
//             _ => {
//                 false
//             }
//         })
//     }
// }

// #[cfg(target_os = "macos")]
// fn uid_to_hw_id(uid: String) -> Result<u32, Error> {
//     let id_property_address = AudioObjectPropertyAddress {
//         mSelector: kAudioHardwarePropertyDeviceForUID,
//         mScope: kAudioDevicePropertyScopeOutput,
//         mElement: kAudioObjectPropertyElementMain,
//     };

//     let cf_uid = CFString::new(&uid);

//     let mut hw_id: u32 = 0;
//     let mut data_size = size_of::<u32>() as u32;
//     unsafe {
//         use std::ptr::{NonNull, null};
//         let hw_id_status = AudioObjectGetPropertyData(kAudioObjectSystemObject as u32,
//             NonNull::new_unchecked(&id_property_address as *const _ as *mut _),
//             size_of::<String>() as u32, cf_uid.as_concrete_TypeRef() as *const c_void,
//             NonNull::new_unchecked(&mut data_size as *mut _ ), NonNull::new_unchecked(&hw_id as *const _ as *mut _));
//         if hw_id_status != 0 {
//             return Err(Error::Placeholder);
//         }
//     }
//     Ok(hw_id)
// }

// #[derive(Debug)]
// pub enum VolumeError {
//     MuteStatusCaptureFailed,
//     UnsupportedOS,
//     ChannelCountCaptureError,
//     DeviceNotFound,
//     VolumeChangeError,
// }

// #[cfg(test)]
// mod tests {
//     use std::str::FromStr;

//     use cpal::traits::{DeviceTrait, HostTrait};

//     use crate::cpal::VolumeControlExt;

//     // use crate::cpal::{VolumeControlExt, uid_to_hw_id};

//     #[test]
//     fn cpal_get_device_name() {
//         use cpal::traits::HostTrait;
//         use crate::cpal::traits::DeviceTrait;
//         let host = cpal::default_host();
//         println!("{:?}", host.id());
//         let device = host.default_output_device().expect("no output device available");
//         println!("{}", device.name().unwrap());
//         // println!("DEVICE UID: {:?}, DEVICE CONVERTED HW_ID: {:?}", device.id(), uid_to_hw_id(device.id().unwrap().1));
//         assert!(false);
//     }

//     #[test]
//     fn cpal_device_vol() {
//         use cpal::traits::HostTrait;
//         use crate::cpal::traits::DeviceTrait;
//         let host = cpal::default_host();
//         let device = host.default_output_device().expect("no output device available");
//         println!("{}", device.name().unwrap());
//         let vol_control =  device.default_volume_control().unwrap();
//         dbg!(vol_control.set_vol(0.20));
//         dbg!(vol_control.get_vol());
//         println!("{:?}", device.default_volume_control().unwrap());
//         assert!(false);
//     }

//     #[test]
//     fn cpal_devices() {
//         let host = cpal::default_host();
//         for device in host.devices().unwrap() {
//             println!("{}", device.name().unwrap_or_default());
//             println!("{:?}", device.id().unwrap());
//         }
//         assert!(false);
//     }

//     // #[test]
//     // fn cpal_from_uid() {
//     //     use cpal::traits::`{HostTrait, DeviceTrait};
//     //     let host = cpal::default_host();
//     //     let device = host.default_output_device().unwrap();
//     //     let vol_control =  device.default_volume_control().unwrap();
//     //     dbg!(vol_control.set_vol(0.20));
//     //     println!("{:?}", device.default_volume_control().unwrap());
//     //     assert!(false);
//     // }
    
// }