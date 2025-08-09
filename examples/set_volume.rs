use cpvc::{get_system_volume, set_system_volume};

fn get_volume() -> u8 {
    let volume = get_system_volume();
    println!("system volume: {volume}");
    volume
}
fn set_volume(volume: u8) -> u8 {
    let previous = get_system_volume();
    println!("setting system volume to: {volume}");
    set_system_volume(volume);
    let current = get_system_volume();
    if previous != current {
        println!("previous system volume: {previous}");
        println!("current system volume: {current}");
    }
    current
}
fn main() {
    get_volume();
    set_volume(0);
    set_volume(50);
    set_volume(100);
}
