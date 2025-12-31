use libpulse_binding::{
    context::{Context, introspect::SinkInfo}, 
    callbacks::ListResult,
    mainloop::standard::Mainloop,
    proplist::Proplist
};
use std::sync::{Arc, Mutex};
use crate::{VolumeControl, debug_eprintln, debug_println, error::Error, pulseaudio::device::PulseAudioDevice};

pub mod device;


// Currently no functionality to detect jacks, only output audio cards
pub struct PulseAudio {}

impl VolumeControl for PulseAudio {
    fn get_sound_devices() -> Result<Vec<String>, Error> {
        Ok(PulseAudio::get_device_identifiers()?.into_iter().map(|(_id, name)| name).collect())
    }

    fn get_vol() -> Result<f32, Error> {
        let default_dev = PulseAudio::get_default_output_dev()?;
        default_dev.get_vol()
    }

    fn set_vol(value: f32) -> Result<(), Error> {
        let default_dev = PulseAudio::get_default_output_dev()?;
        default_dev.set_vol(value)
    }

    fn get_mute() -> Result<bool, Error> {
        let default_dev = PulseAudio::get_default_output_dev()?;
        default_dev.get_mute()
    }

    fn set_mute(state: bool) -> Result<(), Error> {
        let default_dev = PulseAudio::get_default_output_dev()?;
        default_dev.set_mute(state)
    }
}

impl PulseAudio {
    pub fn get_device_identifiers() -> Result<Vec<(String, String)>, Error> {
        let mut devices: Vec<(String, String)> = Vec::new();
        
        let device_list = Arc::new(Mutex::new(Vec::new()));
        let mut mainloop = Mainloop::new().expect("Failed to create mainloop");
        let proplist = Proplist::new().unwrap();
        let mut context = Context::new_with_proplist(&mainloop, "CPVC", &proplist)
            .expect("Failed to create connection context");


        context.connect(None, libpulse_binding::context::FlagSet::NOFLAGS, None)
            .expect("Failed to connect context");

        loop {
            match context.get_state() {
                libpulse_binding::context::State::Ready => break,
                libpulse_binding::context::State::Failed | libpulse_binding::context::State::Terminated => {
                    panic!("Context failed or terminated");
                }
                _ => {
                    mainloop.iterate(false);
                }
            }
        }
        let clone = Arc::clone(&device_list);
        let error = Arc::new(Mutex::new(None));
        let error_clone = error.clone();
        let op = context.introspect().get_sink_info_list(move |info: ListResult<&SinkInfo> | {
            match info {
                libpulse_binding::callbacks::ListResult::Item(device) => {
                    if let Some(description) = device.description.as_ref() && let Some(name) = device.name.as_ref() {
                        if let Ok(mut lock) = clone.lock() {
                            lock.push((name.to_string(), description.to_string()));
                        } else {
                            error_clone.lock().unwrap().replace(Some(Error::DeviceAccessFailed(format!("Failed to unlock device list on device {}", name))));
                        }
                    } else {
                        error_clone.lock().unwrap().replace(Some(Error::DeviceAccessFailed(format!("Failed to access device description"))));
                    }
                },
                libpulse_binding::callbacks::ListResult::End => {
                    debug_println("Devices finished");
                },
                libpulse_binding::callbacks::ListResult::Error => {
                    debug_eprintln("error gathering device information");
                },
            }
        });
        
        while op.get_state() == libpulse_binding::operation::State::Running {
            mainloop.iterate(false);
        }

        mainloop.quit(libpulse_binding::def::Retval(0));

        if let Ok(list) = &mut device_list.lock() {
            devices.append(list);
        } else {
            return Err(Error::DeviceEnumerationFailed(format!("Failed to lock onto device list")));
        }
        Ok(devices)
    }

    pub fn acquire_mainloop_and_context() -> (Mainloop, Context) {
        let mut mainloop = Mainloop::new().expect("Failed to create mainloop");
        let proplist = Proplist::new().unwrap();
        let mut context = Context::new_with_proplist(&mainloop, "CPVC", &proplist)
            .expect("Failed to create connection context");
        
        context.connect(None, libpulse_binding::context::FlagSet::NOFLAGS, None)
            .expect("Failed to connect context");

        loop {
            match context.get_state() {
                libpulse_binding::context::State::Ready => break,
                libpulse_binding::context::State::Failed | libpulse_binding::context::State::Terminated => {
                    panic!("Context failed or terminated");
                }
                _ => {
                    mainloop.iterate(false);
                }
            }
        }

        (mainloop, context)
    }

    // Default output device does not work when Pro Audio is selected as a playback device
    pub fn get_default_output_dev() -> Result<PulseAudioDevice, Error> {
        let default_dev = Arc::new(Mutex::new(String::new()));
        let clone = Arc::clone(&default_dev);

        let (mut mainloop, context) = PulseAudio::acquire_mainloop_and_context();
        let op = context.introspect().get_sink_info_list( move |info | {
                match info {
                    libpulse_binding::callbacks::ListResult::Item(device) => {
                        if let Some(_) = device.active_port {
                            *clone.lock().unwrap() = device.description.as_ref().unwrap().to_string();
                        }
                    },
                    libpulse_binding::callbacks::ListResult::End => {
                        debug_println("Devices finished")
                    },
                    libpulse_binding::callbacks::ListResult::Error => {
                        debug_eprintln("error gathering device information");
                    },
                }
            });
        while op.get_state() == libpulse_binding::operation::State::Running {
            mainloop.iterate(false);
        }
        mainloop.quit(libpulse_binding::def::Retval(0));
        let device_name = default_dev.lock().unwrap().clone();

       

        PulseAudioDevice::from_name(device_name)
    }

    pub fn get_device_id(name: String) -> Result<String, Error> {
        let devices = PulseAudio::get_device_identifiers()?;
        for (dev_str, names) in devices {
            if names == name {
                return Ok(dev_str);
            }
        }
        Err(Error::DeviceNotFound)
    }

    pub fn get_device_name(id: String) -> Result<String, Error> {
        let devices = PulseAudio::get_device_identifiers()?;
        for (dev_str, name) in devices {
            if id == dev_str {
                return Ok(name);
            }
        }
        Err(Error::DeviceNotFound)
    }
}