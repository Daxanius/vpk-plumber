# vpk-plumber
[![Rust](https://github.com/Daxanius/vpk-plumber/actions/workflows/rust.yml/badge.svg?branch=develop)](https://github.com/Daxanius/vpk-plumber/actions/workflows/rust.yml)

vpk-plumber is a fork of [sourcepak](https://github.com/barnabwhy/sourcepak-rs). It provides support for working with [VPK files](https://developer.valvesoftware.com/wiki/VPK_(file_format)). It allows you to read and write VPK files, as well as extract and pack their contents.

## Features
- [x] Read and write VPK files
- [x] Extract files from VPK archives
- [x] Optionally memory-map archive files for faster speeds (with the `mem-map` feature)
- [x] Write VPK directory files (`dir.vpk`)
- [ ] Modify files in VPK archives
- [ ] Add files to VPK archives
- [ ] Remove files from VPK archives

## Supported formats
### VPK v1 (Alien Swarm, Dota 2, L4D, L4D2, Portal 2, SFM)
- [x] Read directory files
- [x] Read file contents
- [x] Write VPK directories
- [ ] Modify files in VPK archives
- [ ] Add files to VPK archives
- [ ] Remove files from VPK archives

### VPK v2 (CS:GO, CS:S, DoD:S, HL:S, HL2, HL2:DM, Portal, TF2, Source 2)
- [x] Read directory files
- [ ] Read file contents
- [ ] Write VPK directories
- [ ] Modify files in VPK archives
- [ ] Add files to VPK archives
- [ ] Remove files from VPK archives

### Respawn VPK (Titanfall)
- [x] Read directory files
- [x] Read file contents
- [x] Read audio files (see [here](https://github.com/barnabwhy/TF1.Audio.English?tab=readme-ov-file#why-did-respawn-decompress-the-audio-in-the-first-place) for why this is separate)
- [x] Write VPK directories
- [ ] Modify files in VPK archives
- [ ] Add files to VPK archives
- [ ] Remove files from VPK archives

## Why does this fork exist?
sourcepak is a fantastic crate providing a good structure and features to handle VPK files. However, it still lacks support for some operations and features that I want to make use of. For this reason, I forked the project in order to add those features myself.
