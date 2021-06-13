# mtpcopy

A Windows CLI tool for copying files from/to a portable device through MTP.

This tool calls [Windows Portable Devices API](https://docs.microsoft.com/en-us/windows/win32/wpd_sdk/wpd-application-programming-interface)
using [Rust for Windows](https://github.com/microsoft/windows-rs).

## Examples

### Copy a local folder to a portable device

```sh
mtpcopy copy -R ".\My Music" "My Device:SD Card:\Data"
```

* command: `copy`
* flags: `-R` (recursive)
* source path: `.\My Music`
* destination path: `My Device:SD Card:\Data`
   * device name: `My Device`
   * storage name: `SD Card`
   * path on the storage: `\Data`

### Copy a folder from a portable device

```sh
mtpcopy copy -R "My Device:SD Card:\Data\My Music\Tilt" "D:\My Music"
```

* command: `copy`
* flags: `-R` (recursive)
* source path: `My Device:SD Card:\Data\My Music\Tilt`
   * device name: `My Device`
   * storage name: `SD Card`
   * path on the storage: `\Data\My Music\Tilt`
* destination path: `D:\My Music` (`D:\My Music\Tilt` will be created if it doesn't exist)

### List portable device storages

```sh
mtpcopy storages
```

* command: `storages`

### List files on the portable device storages

```sh
mtpcopy list -R "My Device:SD Card:\Data\My Music"
```

* command: `list`
* flags: `-R` (recursive)
* path: `My Device:SD Card:\Data\My Music`
   * device name: `My Device`
   * storage name: `SD Card`
   * path on the storage: `\Data\My Music`

### List files on the portable device storages with the Glob pattern

```sh
mtpcopy list "*:SD*:\Pictures\202?\**\*.jpg"
```

* command: `list`
* path: `*:SD*:\Pictures\202?\**\*.jpg`
   * device name: `*` (all devices)
   * storage name: `SD*` (starts with `SD`)
   * path on the storage: `\Pictures\202?\**\*.jpg` (any jpg files )
