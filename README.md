# Bangor (Secure Voice Communication Platform)

**Bangor** is a lightweight, secure, and modular voice communication platform built in **Rust**, optimized for **low-latency, encrypted audio streaming** on devices like the **Raspberry Pi 4B**.

The project is inspired by SFU-style architectures (like Discord), using **centralized UDP relaying** for real-time audio and a secure **TCP control channel** for commands, login, and room management.

The goal of this project is to let me explore principles such as encryption, networking, databases, interfacing with hardware devices, and Rust. The project as a whole right now is bare as I just started moving from a proof-of-concept build which interface directly with a raspi, to something scalable with a database and proper user auth. I am cureently learning about databases and taking my time designing the database structure to be efficient and simple. There is also Bangor-Server repo on my github as the serverside applicaiton. 

---

## Features

- [x] Encrypted UDP audio streaming using **Opus** and **Libsodium (XSalsa20-Poly1305)**
- [x] Lightweight architecture designed for **embedded servers (e.g. Raspberry Pi)**
- [x] Cross-platform audio support via [`cpal`] for input/output
- [x] Secure control channel using **Noise Protocol (`XX`)**
- [x] Username + password authentication over encrypted TCP(basic w/ no database)
- [x] Modular design (audio, crypto, control separated cleanly)
-> [ ] SQL database using PostGreSQL
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

| Component  | Crate         | Notes                                              |
| ---------- | ------------- | ----------------------------                       |
| Audio I/O  | `cpal`        | Cross-platform mic/speaker device driver library   |
| Codec      | `opus`        | Opus encoder/decoder binding                       |
| Crypto     | `sodiumoxide` | Libsodium wrapper                                  |
| Handshake  | `snow`        | Noise Protocol framework                           |
| Networking | `std::net`    | Raw UDP and TCP                                    |
