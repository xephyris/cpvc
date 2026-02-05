# CPVC Command

CPVC Command library uses `std::process::Command` and the systems local tools for the same feature set instead of built in Rust crates.

## Submodule Status
>[!CAUTION]  
> Due to the maturity of `cpvc`, `cpvc::command` is no longer maintained and will not be receiving updates. \
Only critical issues will be resolved. `cpvc` is the recommended replacement with similar functionality.


## Important User Details

> [!IMPORTANT]  
> `cpvc::command` on Linux requires a system using ALSA and PipeWire, due to the commands used. \
> Support for other audio drivers may be implemented in the future.

> [!WARNING]  
> `cpvc::command` does NOT support Windows. \
> Windows does not allow have powershell commands to gather audio information.\
> Some functions may be implemented, but use at your own risk.

## Tested/Worked On
* macOS:
   * Sequoia 15.1

* Linux:
   * EndeavourOS Mercury

## Development Details

`cpvc::command` uses these progams for macOS and Linux.

* macOS: `osascript`
* Linux: `amixer` and `pw-cli`
