## Cross Platform Volume Control (CPVC)

Basic cross platform crate for interacting with Audio Devices and handling System Audio

## Important User Details

> [!IMPORTANT]  
> `cpvc` requires PulseAudio server to work on Linux. \
> This is due to the crates used to interact with system APIs. \
> For more information scroll below.

> [!IMPORTANT]  
> If you want `cpvc` to print possible non critical errors, 
> enable the debug feature when adding the crate.


## Tested/Worked On
* macOS:
   * Sequoia 15.5

* Windows:
    * Windows 11 24H2

* Linux:
   * EndeavourOS Mercury

## Development Details

`cpvc` uses these crates for each platform.

* macOS:
    * `objc2_core-audio`
    * `objc2-core-audio-types`
    * `core-foundation`
* windows: 
    * `windows`
* Linux: 
    * `libpulse-binding`
    * `libpulse-sys`


### Why only PulseAudio?
Unfortunately, at the moment, there are not any viable crates that are as comprehensive as `libpulse-binding` that I have found to support all the features `cpvc` requires. 

If you want to contribute code for another audio API, feel free to submit a pull request!



