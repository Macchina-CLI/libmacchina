# Changelog

## `8.0.0`

Rolv Apneseth:
  - BREAKING CHANGE: Allow disk_space function to accept a path argument (#156)

## `7.0.0`

Rolv Apneseth:
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
