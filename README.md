# sourcepak
sourcepak is a Rust crate that provides support for working with VPK (Valve Pak) files. It allows you to read and write VPK files, as well as extract and pack their contents.

## Features
- [x] Read and write VPK files
- [x] Extract files from VPK archives
- [ ] Pack files into VPK archives

## Supported formats
### VPK v1
- [x] Read directory files
- [x] Read file contents
- [ ] Patch existing VPKs
- [ ] Write new VPKs

### VPK v2
- [x] Read directory files
- [ ] Read file contents
- [ ] Patch existing VPKs
- [ ] Write new VPKs

### Respawn VPK
- [x] Read directory files
- [x] Read file contents
- [ ] Patch existing VPKs
- [ ] Write new VPKs

## Documentation
Coming soon

## Why does this crate exist?
I originally created the [TFVPKTool](https://github.com/barnabwhy/TFVPKTool) TypeScript library to support reading Respawn VPK files, along with [Harmony VPK Tool](https://github.com/harmonytf/HarmonyVPKTool) using Electron.

I very quickly noticed the issue that these often resulted in high memory use due to language and ecosystem I had used.

With sourcepak I am aiming to fix that.