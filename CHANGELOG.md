# Changelog

## `7.3.1`

- Fix overflow affecting linux::count_homebrew() implementation

## `7.3.0`

coolGi2007:
- Add support for the Nix package manager
- Bump `sqlite` dependency to version `0.36.0`
- Don't panic if the `pci.ids` database could not be found

Rex Ng:
- Recognize latest version of macOS

## `7.2.3`

- Fix `Readouts` struct `network` field type

Matthias Baer:
- Improve CI workflow (#169)

Adrian Groh:
- Faster package count on Alpine Linux (#170)

## `7.2.2`

- Update `vergen` dependency and correct its invocation.

## `7.2.1`

- Fix some build errors

## `7.2.0`

Adrian Groh:
- Fix build errors affecting i686 (#162)

Rex Ng:
- Recognize macOS Sonoma (#163)

Absolpega:
- Add general detection for Wayland (#164)

- Change the return value of BatteryReadout::health from `u64` to `u8`
- Upgrade dependencies to their latest versions

## `7.1.0`

Silas Groh:
  - Add support for `ndb` databases

Adrian Groh:
  - Replace `mach` dependency with `mach2`
  - Replace `python` command with `sh` in `extra::which` unit tests
  - Add armv7 to the list of build targets in the CI pipeline
  - Fix compilation issues for armv7 build target

## `7.0.0`

- Rolv Apneseth:
  - BREAKING CHANGE: Change disk_space return value to u64 (#153)

## `6.4.1`

- Default to GPU device name if subdevice name is not found
- Detect VGA compatible controllers
- Correctly filter battery devices when retrieving their status

## `6.4.0`

Adrian Groh:
  - Use the correct kernel parameters when initializing FreeBSD `KernelReadout` (#148)
  - Implement uptime readout for FreeBSD systems (#138)
  - Use `MemAvailable` to calculate used memory (#134)
  - Prioritize detecting window managers with xprop (#133)

Rolv Apneseth: Implement GPU readout for Linux systems (#140)

Matthias Baer: Use a singleton for `COMLibrary` (#143)

Xarblu: Change Flatpak package-counting method (#125)

Kian-Meng Ang: Fix a typo in the documentation

## `6.3.5`

- Ignore clippy unnecessary_cast warnings in shared module

## `6.3.4`

- Add missing bang in clippy allow macros

## `6.3.3`

- Ignore clippy unnecessary_cast warnings

## `6.3.2`

@TheCactusVert:
- Fix cargo package count (#128)

@123marvin123 and @Markos-Th09:
- Fix brew package count (#127)

@xarblu:
- Fix portage package count (#124)

## `6.3.1`

@123marvin123:
- Fix a bug that returns a nil framerate on certain macOS systems

## `6.3.0`

@123marvin123:
- Implement backlight readout for macOS

## `6.2.0`

- Update dependencies where needed, bringing us up to speed with the
  latest and greatest stuff from the libraries we use.

@DemonInTheCloset:
- Fix armv7 compilation issues
