use std::net::SocketAddr;
use std::sync::Mutex; // Keep Mutex for CallMap and LOCATION_ENTRIES

// --- Constants ---
pub const BUFFER_SIZE: usize = 1400;
pub const MAX_THREADS: usize = 5;
pub const QUEUE_CAPACITY: usize = 10;
pub const SIP_PORT: u16 = 5060;
pub const MAX_CALLS: usize = 32;
pub const MAX_UUID_LENGTH: usize = 128;
pub const MAX_USERNAME_LENGTH: usize = 16;
pub const DEFAULT_MAX_FORWARDS: u32 = 70;
pub const REGISTER_CONTACT_EXPIRES: u32 = 7200;
pub const RPORT_FLAG_VALUE: u16 = 0;

// NOTE: Set this to your server's actual IP address!
pub const SIP_SERVER_IP_ADDRESS: &str = "192.168.32.131"; // Example, change as needed

// Define a-leg and b-leg constants
pub const A_LEG: i32 = 1;
pub const B_LEG: i32 = 2;

// Define message type constants
pub const REQUEST_METHOD: i32 = 1;
pub const STATUS_CODE: i32 = 2;

// --- Structs ---

// Holds received message and client address
#[derive(Debug, Clone)]
pub struct SipMessage {
    pub buffer: Vec<u8>, // Use Vec<u8> for raw bytes
    pub client_addr: SocketAddr,
}

// User location information
#[derive(Debug, Clone)]
pub struct LocationEntry {
    pub username: String,
    pub ip_str: String, // Keep as String for consistency with C
    pub port: u16,
    pub registered: bool,
    // Runtime address (updated on REGISTER)
    pub current_addr: Option<SocketAddr>,
}

// Media state for a leg
#[derive(Debug, Clone, Copy, Default)]
pub struct MediaState {
    pub local_media: bool,
    pub remote_media: bool,
}

// Call states enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CallState {
    #[default]
    Idle,
    Routing,
    Ringing,
    Answered,
    Connected,
    Disconnecting,
}

// Key SIP headers for a leg
#[derive(Debug, Clone, Default)]
pub struct SipHeaderInfo {
    pub from: String,
    pub via: String,
    pub cseq: String,
    pub to: String,
    // Add others as needed, e.g., Contact
}

// Represents an ongoing call
#[derive(Debug, Clone, Default)]
pub struct Call {
    pub a_leg_uuid: String,
    pub b_leg_uuid: String,
    pub call_state: CallState,
    pub a_leg_media: MediaState,
    pub b_leg_media: MediaState,
    pub a_leg_addr: Option<SocketAddr>, // Store Option<SocketAddr> directly
    pub b_leg_addr: Option<SocketAddr>, // Store Option<SocketAddr> directly
    pub index: usize,                   // Index within the CallMap's Vec
    pub a_leg_header: SipHeaderInfo,
    pub b_leg_header: SipHeaderInfo,
    pub callee: String,        // Max 32 in C
    pub a_leg_contact: String, // Store full contact header or parsed URI
    pub b_leg_contact: String, // Store full contact header or parsed URI
    pub is_active: bool,
    // Mutex per call removed as requested; access controlled by CallMap's Mutex
}

// Manages all active calls
#[derive(Debug)]
pub struct CallMap {
    pub calls: Vec<Call>, // Use a Vec, manage is_active flag
    pub size: usize,      // Number of active calls
                          // Mutex moved here, wraps the entire CallMap
}

// --- Static Data ---
use lazy_static::lazy_static;

// NOTE: Define your user entries here, similar to the C code.
// IPs/Ports in the static definition are defaults; `current_addr` is updated by REGISTER.
lazy_static! {
    pub static ref LOCATION_ENTRIES: Mutex<Vec<LocationEntry>> = Mutex::new(vec![
        LocationEntry { username: "1001".to_string(), ip_str: "192.168.32.10".to_string(), port: 5060, registered: false, current_addr: None },
        LocationEntry { username: "1002".to_string(), ip_str: "192.168.32.10".to_string(), port: 5070, registered: false, current_addr: None },
        LocationEntry { username: "1003".to_string(), ip_str: "192.168.1.103".to_string(), port: 5060, registered: false, current_addr: None },
        LocationEntry { username: "1004".to_string(), ip_str: "192.168.1.104".to_string(), port: 5060, registered: false, current_addr: None },
        LocationEntry { username: "1005".to_string(), ip_str: "192.168.184.1".to_string(), port: 5060, registered: false, current_addr: None },
        LocationEntry { username: "1006".to_string(), ip_str: "192.168.184.1".to_string(), port: 5070, registered: false, current_addr: None },
        // Add more users as needed
    ]);
}

// --- Global CSeq ---
// Use atomic for thread-safe incrementing without a full mutex just for this counter.
use std::sync::atomic::{AtomicUsize, Ordering};
pub static CSEQ_NUMBER: AtomicUsize = AtomicUsize::new(1);

// Helper to get next CSeq
pub fn next_cseq() -> usize {
    CSEQ_NUMBER.fetch_add(1, Ordering::SeqCst)
}

// Helper function to update location entry's address and registration status
// Returns true if update was successful, false if user not found
pub fn update_location_entry_addr(username: &str, addr: SocketAddr) -> bool {
    let mut entries = match LOCATION_ENTRIES.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!(
                "LOCATION_ENTRIES mutex poisoned while updating; continuing with existing data."
            );
            poisoned.into_inner()
        }
    };
    if let Some(entry) = entries.iter_mut().find(|entry| entry.username == username) {
        entry.current_addr = Some(addr);
        // Update ip_str and port as well, based on the received addr for consistency?
        entry.ip_str = addr.ip().to_string();
        entry.port = addr.port();
        entry.registered = true;
        println!(
            "User {} registered successfully from {}",
            entry.username, addr
        );
        println!(
            "Location entry for user '{}' updated to IP: {}, Port: {}",
            entry.username, entry.ip_str, entry.port
        );
        true
    } else {
        false
    }
}

// Helper function to get a registered user's current address
pub fn get_registered_addr(username: &str) -> Option<SocketAddr> {
    let entries = match LOCATION_ENTRIES.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("LOCATION_ENTRIES mutex poisoned while reading; returning last known state.");
            poisoned.into_inner()
        }
    };
    entries
        .iter()
        .find(|entry| entry.username == username && entry.registered)
        .and_then(|entry| entry.current_addr) // Return the Option<SocketAddr>
}
