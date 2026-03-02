# Whisper Desktop

End-to-end encrypted messaging for people who actually care about privacy. No cloud, no tracking, no "trust us" handwaving.

## Why This Exists

Signal is great but requires a phone number. Matrix is federated but complex. Most "private" messengers still route everything through servers that could theoretically read your messages if sufficiently motivated.

Whisper takes a different approach: your keys never leave your device, messages are encrypted before transmission, and the relay server is cryptographically incapable of reading anything. Even if someone compromises the relay, they get meaningless ciphertext.

## Features

- End-to-end encryption with XChaCha20-Poly1305
- Ed25519 identity keys (the same crypto as Signal, but we actually let you verify)
- Zero-knowledge relay (the server literally cannot read your messages)
- SQLCipher encrypted local storage (your database is locked with your passphrase)
- System tray integration (lives quietly in the corner)
- Native notifications (without leaking message content to the OS if you don't want)
- Cross-device sync via QR code linking
- Auto-start, auto-lock, and all the quality-of-life features you'd expect

## Security Model

Your passphrase encrypts your secret key using Argon2id. The encrypted key is stored locally in a SQLCipher database. When you unlock, the key is held in memory only. Lock the app, key gone.

Messages are encrypted with ephemeral X25519 key exchange, so even if someone gets your long-term key later, past messages remain secure (forward secrecy). The relay only sees: sender peer ID, recipient peer ID, encrypted blob, timestamp. That's it.

## Quick Start

```bash
# Install dependencies
npm install

# Run in development
npm run tauri dev

# Build for production
npm run tauri build
```

Requires:
- Rust 1.75+
- Node.js 20+
- Platform deps: Xcode (macOS), Visual Studio Build Tools (Windows), webkit2gtk (Linux)

## Related Projects

Whisper is a family of clients sharing the same protocol:

- [whisper](https://github.com/sudokatie/whisper) - Terminal client (Rust)
- [whisper-mobile](https://github.com/sudokatie/whisper-mobile) - Progressive web app
- [whisper-native](https://github.com/sudokatie/whisper-native) - React Native client
- [whisper-relay](https://github.com/sudokatie/whisper-relay) - Message relay server

## Philosophy

1. Your keys, your device, your messages
2. Zero trust architecture (not "trust us" architecture)
3. Open source everything
4. Simple enough to audit, robust enough to use

## License

MIT

## Author

Katie

---

*Your conversations belong to you.*
