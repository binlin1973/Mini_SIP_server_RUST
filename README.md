# Mini_SIP_server (Rust)

[![CI](https://github.com/binlin1973/Mini_SIP_server_RUST/actions/workflows/ci.yml/badge.svg)](https://github.com/binlin1973/Mini_SIP_server_RUST/actions/workflows/ci.yml)

A **lightweight SIP signaling server** written in Rust.  
Implements the complete SIP call flow — from the initial `INVITE` and `180 Ringing`, to `200 OK`, `ACK`, and `BYE`.  
It focuses purely on SIP **signaling and call control**, not RTP media forwarding.

---

## 🧩 Overview

`Mini_SIP_server` is designed for learning, testing, and integration purposes.  
It lets any standard SIP softphone (e.g., **Linphone**, **MicroSIP**, **Zoiper**) register and make peer-to-peer calls.

The server runs as a single lightweight binary and stores user registration information in memory.

---

## ⚙️ Build & Run

### 1. Configure Server IP

Before building, open [`sip_defs.rs`](./src/sip_defs.rs) and set your actual server IP address:
pub const SIP_SERVER_IP_ADDRESS: &str = "192.168.32.131"; // Example — change to your machine's IP

### 2. Build

cargo clean
cargo build --release --target x86_64-unknown-linux-musl

### 3. Run
./target/x86_64-unknown-linux-musl/release/sip_server_rust
By default, the server listens on UDP port 5060.


## ⚙️ Softphone Configuration
Any standard SIP softphone can connect to this server.

Setting	Example	Description
SIP Server / Proxy	      192.168.32.131	       Replace with your server IP
Port	                  5060	                   Default UDP port
Username	              1001 – 1006	           Any user ID in this range
Password	              any non-empty string	   Password is not validated
Transport	              UDP	                   Required

Example (MicroSIP)
Field	                  Value
Account name	          1001
SIP server	              192.168.32.131
User	                  1001
Domain	                  192.168.32.131
Password	              1234
Transport	              UDP

##  📞 Making a Call

Register two clients, e.g.:
Client A → 1001
Client B → 1002

From Client A, dial 1002

Client B will ring and can answer the call.
You’ll see the full SIP signaling printed in the server console:
INVITE → 100 Trying → 180 Ringing → 200 OK → ACK → BYE → 200 OK

##  🧠 Internal State Machine
For a deeper understanding of how SIP states transition through the call lifecycle,
see State_Machine_Design.pdf

##  🧪 run tests
cargo test --all

##  License
MIT License © Bin Lin
Lightweight, educational, and open to contributions.
