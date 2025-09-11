pub extern crate cpal;

trait VolumeControlExt {
    fn set_volume(&self, vol: f32) -> bool;
    
    fn get_volume(&self) -> f32;
    
    fn set_mute(&self, mute: bool) -> bool;
    
    fn is_mute(&self) -> bool;
}

// impl VolumeControlExt for cpal::Device {
//     fn set_volume(&self, vol: f32) -> bool {
//         todo!()
//     }

//     fn get_volume(&self) -> f32 {
//         todo!()
//     }

//     fn set_mute(&self, mute: bool) -> bool {
//         todo!()
//     }

//     fn is_mute(&self) -> bool {
//         todo!()
//     }
// };

#[cfg(test)]
mod tests {
    use cpal::traits::HostTrait;

    #[test]
    fn cpal_get_device_name() {
        use cpal::traits::HostTrait;
        use crate::cpal::cpal::traits::DeviceTrait;
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available");
        println!("{}", device.name().unwrap());
        assert!(false);
    }
    
}