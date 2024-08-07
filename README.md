# sourcepak
sourcepak is a Rust crate that provides support for working with VPK (Valve Pak) files. It allows you to read and write VPK files, as well as extract and pack their contents.

[![crates.io](https://img.shields.io/crates/v/sourcepak.svg)](https://crates.io/crates/sourcepak)

## Features
- [x] Read and write VPK files
- [x] Extract files from VPK archives
- [x] Optionally memory-map archive files for faster speeds (with the `mem-map` feature)
- [x] Write VPK directory files (`dir.vpk`)

## Supported formats
### VPK v1 (Alien Swarm, Dota 2, L4D, L4D2, Portal 2, SFM)
- [x] Read directory files
- [x] Read file contents
- [x] Write VPK directories

### VPK v2 (CS:GO, CS:S, DoD:S, HL:S, HL2, HL2:DM, Portal, TF2, Source 2)
- [x] Read directory files
- [ ] Read file contents
- [ ] Write VPK directories

### Respawn VPK (Titanfall)
- [x] Read directory files
- [x] Read file contents
- [x] Read audio files (see [here](https://github.com/barnabwhy/TF1.Audio.English?tab=readme-ov-file#why-did-respawn-decompress-the-audio-in-the-first-place) for why this is separate)
- [x] Write VPK directories

## Documentation
Documentation can be found [here](https://docs.rs/sourcepak)

## Why does this crate exist?
I originally created the [TFVPKTool](https://github.com/barnabwhy/TFVPKTool) TypeScript library to support reading Respawn VPK files, along with [Harmony VPK Tool](https://github.com/harmonytf/HarmonyVPKTool) using Electron.

I very quickly noticed the issue that these often resulted in high memory use due to language and ecosystem I had used.

With sourcepak I am aiming to fix that.
