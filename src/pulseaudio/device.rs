
use std::sync::{Arc, Mutex};
use crate::{debug_eprintln, debug_println, error::Error, pulseaudio::PulseAudio};

pub struct PulseAudioDevice {
    dev_str: String,
}

impl PulseAudioDevice {

    pub fn from_name(name: String) -> Result<Self, Error> {
        let devices = PulseAudio::get_device_identifiers()?;

        for (id, names) in devices {
            if name == names {
                return Ok(PulseAudioDevice {
                    dev_str: id,
                });
            }
        }
        
        Err(Error::DeviceNotFound)

    }

    pub fn from_id(id: String) -> Result<Self, Error> {
        let devices = PulseAudio::get_device_identifiers()?;

        for (dev_str, _name) in devices {
            if id == dev_str {
                return Ok(PulseAudioDevice {
                    dev_str,
                });
            }
        }
        
        Err(Error::DeviceNotFound)
    }

    pub fn get_name(&self) -> Result<String, Error> {
        PulseAudio::get_device_name(self.dev_str.clone())
    }

    pub fn get_device_str(&self) -> String {
        self.dev_str.clone()
    }


    pub fn get_vol(&self) -> Result<f32, Error> {
        let mut vol = 0.0;
        let volume: Arc<Mutex<f32>> = Arc::new(Mutex::new(vol));
        let clone = Arc::clone(&volume);
        let dev_str = self.dev_str.clone();
        let (mut mainloop, context) = PulseAudio::acquire_mainloop_and_context();

        let changed = Arc::new(Mutex::new(false));
        let changed_clone = changed.clone();

        let error = Arc::new(Mutex::new(None));
        let err_clone = error.clone();

        let op = context.introspect().get_sink_info_list( move |info | {
                match info {
                    libpulse_binding::callbacks::ListResult::Item(device) => {
                        if let Some(name) = &device.name && name.to_string() == dev_str{
                            if device.mute {
                                *clone.lock().unwrap() = 0.0;
                            } else {
                                let mut vol_str = device.volume.avg().print().trim().to_string();
                                vol_str.remove(vol_str.len() - 1);
                                match vol_str.parse::<u8>() {
                                    Ok(vol) => {
                                        *clone.lock().unwrap() = vol as f32 / 100.0;
                                    },
                                    Err(err) => {
                                        err_clone.lock().unwrap().replace(Error::VolumeCaptureFailed(format!("Failed to parse volume string {}", err))); 
                                        debug_eprintln(&format!("Failed to parse volume string {}", err));
                                    }
                                }
                            }
                            *changed_clone.lock().unwrap() = true;
                        }
                    },
                    libpulse_binding::callbacks::ListResult::End => {
                        debug_println("Devices finished")
                    },
                    libpulse_binding::callbacks::ListResult::Error => {
                        err_clone.lock().unwrap().replace(Error::VolumeCaptureFailed(format!("ListResult Access Error"))); 
                        debug_eprintln("error gathering device information");
                    },
                }
            });
        
        while op.get_state() == libpulse_binding::operation::State::Running {
            mainloop.iterate(false);
        }

        mainloop.quit(libpulse_binding::def::Retval(0));

        vol = *volume.lock().unwrap();
        if let Some(error) = error.lock().unwrap().take() {
            Err(error)
        } else if *changed.lock().unwrap() {
            Ok(vol)
        } else {
            Err(Error::VolumeCaptureFailed(format!("Failed to detect device")))
        }
    }

    pub fn set_vol(&self, value: f32) -> Result<(), Error> {
        let mut success = None;
        let vol_channels = Arc::new(Mutex::new(None));
        let clone = Arc::clone(&vol_channels);
        let dev_str = self.dev_str.clone();

        let changed = Arc::new(Mutex::new(false));
        let changed_clone = changed.clone();

        let error = Arc::new(Mutex::new(None));
        let err_clone = error.clone();

        let (mut mainloop, context) = PulseAudio::acquire_mainloop_and_context();
        let op = context.introspect().get_sink_info_list( move |info | {
                match info {
                    libpulse_binding::callbacks::ListResult::Item(device) => {
                        if let Some(name) = &device.name && name.to_string() == dev_str {
                            use libpulse_binding::volume::{Volume};
                            use libpulse_sys::volume::PA_VOLUME_NORM;
                            let vol = Volume((value * PA_VOLUME_NORM as f32) as u32);
                            let mut channel_vols = device.volume;
                            channel_vols.set(device.sample_spec.channels, vol.into());
                            clone.lock().unwrap().replace((device.index, channel_vols));
                            *changed_clone.lock().unwrap() = true;
                        }
                    },
                    libpulse_binding::callbacks::ListResult::End => {
                        debug_println("channel volume aquired");
                    },
                    libpulse_binding::callbacks::ListResult::Error => {
                        err_clone.lock().unwrap().replace(Error::VolumeCaptureFailed(format!("ListResult Access Error"))); 
                        debug_eprintln("error gathering device information");    
                    },
                }
            });

        
        while op.get_state() == libpulse_binding::operation::State::Running {
            mainloop.iterate(false);
        }

        if let Some((index, volume)) = vol_channels.lock().unwrap().take() {
            let vol_runner;
            if value == 0.0 {
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

        if let Some(error) = error.lock().unwrap().take() {
            Err(error)
        } else if !*changed.lock().unwrap() {
            Err(Error::VolumeCaptureFailed(format!("Failed to detect device")))
        } else {
            match success {
                Some(val) => {
                    if val {
                        Ok(())
                    } else {
                        Err(Error::VolumeCaptureFailed(format!("Failed to adjust device volume")))
                    }
                }
                None => {
                    Err(Error::VolumeCaptureFailed(format!("Failed to adjust device volume")))
                }
            }
        }
    }

    pub fn get_mute(&self) -> Result<bool, Error> {
        let mute;
        let mute_status = Arc::new(Mutex::new(0));
        let clone = Arc::clone(&mute_status);
        let dev_str = self.dev_str.clone();

        let (mut mainloop, context) = PulseAudio::acquire_mainloop_and_context();

        let changed = Arc::new(Mutex::new(false));
        let changed_clone = changed.clone();

        let error = Arc::new(Mutex::new(None));
        let err_clone = error.clone();

        let op = context.introspect().get_sink_info_list( move |info | {
                match info {
                    libpulse_binding::callbacks::ListResult::Item(device) => {
                        
                        if let Some(name) = &device.name && name.to_string() == dev_str {
                            if device.mute {
                                *clone.lock().unwrap() = 1;
                            }
                            *changed_clone.lock().unwrap() = true;
                        }
                    },
                    libpulse_binding::callbacks::ListResult::End => {
                        debug_println("Devices finished")
                    },
                    libpulse_binding::callbacks::ListResult::Error => {
                        err_clone.lock().unwrap().replace(Error::VolumeCaptureFailed(format!("ListResult Access Error"))); 
                        debug_eprintln("error gathering device information");
                    },
                }
            });
        
        while op.get_state() == libpulse_binding::operation::State::Running {
            mainloop.iterate(false);
        }

        mainloop.quit(libpulse_binding::def::Retval(0));

        mute = *mute_status.lock().unwrap();
        if let Some(error) = error.lock().unwrap().take() {
            Err(error)
        } else if !*changed.lock().unwrap() {
            Err(Error::VolumeCaptureFailed(format!("Failed to detect device")))
        } else {
            match mute {
                1 => {
                    Ok(true)
                }
                _ => {
                    Ok(false)
                }
            }
        }
    }

    pub fn set_mute(&self, mute: bool) -> Result<(), Error> {
        let mut status = false;
        let dev_index = Arc::new(Mutex::new(None));
        let dev_str = self.dev_str.clone();
        let clone = Arc::clone(&dev_index);

        let (mut mainloop, context) = PulseAudio::acquire_mainloop_and_context();

        let changed = Arc::new(Mutex::new(false));
        let changed_clone = changed.clone();

        let error = Arc::new(Mutex::new(None));
        let err_clone = error.clone();

        let op = context.introspect().get_sink_info_list( move |info | {
                match info {
                    libpulse_binding::callbacks::ListResult::Item(device) => {
                        if let Some(name) = &device.name && name.to_string() == dev_str{
                            clone.lock().unwrap().replace(device.index);
                            *changed_clone.lock().unwrap() = true;
                        }
                    },
                    libpulse_binding::callbacks::ListResult::End => {
                        debug_println("channel volume aquired");
                    },
                    libpulse_binding::callbacks::ListResult::Error => {
                        err_clone.lock().unwrap().replace(Error::VolumeCaptureFailed(format!("ListResult Access Error"))); 
                        debug_eprintln("error gathering device information");    
                    },
                }
            });

        
        while op.get_state() == libpulse_binding::operation::State::Running {
            mainloop.iterate(false);
        }


        if let Some(index) = dev_index.lock().unwrap().take() {
            let mute_runner;
            mute_runner = context.introspect().set_sink_mute_by_index(index, mute, None);        
            while mute_runner.get_state() == libpulse_binding::operation::State::Running {
                mainloop.iterate(false);
                status = true;
            }
        } else {
            status = false;
        }
        mainloop.quit(libpulse_binding::def::Retval(0));
        if let Some(error) = error.lock().unwrap().take() {
            Err(error)
        } else if !*changed.lock().unwrap() {
            Err(Error::VolumeCaptureFailed(format!("Failed to detect device")))
        } else {
            match status {
                true => {
                    Ok(())
                }
                false => {
                    Err(Error::VolumeCaptureFailed(format!("Failed to adjust device volume")))
                }
            }
        }
    } 
}
