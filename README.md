# RankedDiscord (Secure Voice Communication Platform)

**RankedDiscord** is a lightweight, secure, and modular voice communication platform built in **Rust**, optimized for **low-latency, encrypted audio streaming** on devices like the **Raspberry Pi 4B**.

The project is inspired by SFU-style architectures (like Discord), using **centralized UDP relaying** for real-time audio and a secure **TCP control channel** for commands, login, and room management.

---

## Features

- [x] Encrypted UDP audio streaming using **Opus** and **Libsodium (XSalsa20-Poly1305)**
- [x] Lightweight architecture designed for **embedded servers (e.g. Raspberry Pi)**
- [x] Cross-platform audio support via [`cpal`] for input/output
- [x] Secure control channel using **Noise Protocol (`XX`)**
- [x] Username + password authentication over encrypted TCP
- [x] Modular design (audio, crypto, control separated cleanly)
- [ ] Room-based audio routing
- [ ] Optional Opus bitrate control + voice activity detection
- [ ] CLI or TUI client interface
- [ ] Optional TLS upgrade or WebRTC bridge

---

## Architecture Overview

```plaintext
                      +-------------------+
                      |     TCP (Noise)   |
                      | Login / Control   |
                      +-------------------+
                               |
+--------+    UDP (Opus + Libsodium)     +--------+
| Client | <---------------------------> | Server |
+--------+                              +--------+
                               |
                      +-------------------+
                      | UDP Relay to Peers |
                      +-------------------+

```
## Stack

| Component  | Crate         | Notes                        |
| ---------- | ------------- | ---------------------------- |
| Audio I/O  | `cpal`        | Cross-platform mic/speaker   |
| Codec      | `opus`        | Opus encoder/decoder binding |
| Crypto     | `sodiumoxide` | Libsodium wrapper            |
| Handshake  | `snow`        | Noise Protocol framework     |
| Networking | `std::net`    | Raw UDP and TCP              |
