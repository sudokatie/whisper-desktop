# Whisper Desktop

Native desktop client for Whisper encrypted messaging. Because your conversations should stay yours.

## Status

**Work in Progress** - Foundation and build docs complete, implementation starting.

## What It Does

End-to-end encrypted messaging that syncs with your mobile devices. Built with Tauri (Rust + React), sharing crypto primitives with the Whisper ecosystem.

- Ed25519 identity keys
- XChaCha20-Poly1305 encryption  
- Zero-knowledge relay (server cannot read messages)
- System tray integration
- Native notifications
- Cross-device sync

## Related Projects

- [whisper](https://github.com/sudokatie/whisper) - TUI client (Rust)
- [whisper-mobile](https://github.com/sudokatie/whisper-mobile) - PWA client
- [whisper-native](https://github.com/sudokatie/whisper-native) - React Native client
- [whisper-relay](https://github.com/sudokatie/whisper-relay) - Message relay server

## Development

Requires:
- Rust 1.75+
- Node.js 20+
- Tauri CLI: `cargo install tauri-cli`

```bash
# Install dependencies
npm install

# Run development server
npm run tauri dev

# Build for production
npm run tauri build
```

## License

MIT

## Author

Katie
