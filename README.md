# Mini_SIP_server (RUST)
A lightweight SIP server written in RUST, implementing the full SIP call flow—from the initial `INVITE` through interim states to the final `200 OK` response for `BYE`. Focuses solely on call setup and teardown.

### NOTE 1: Server IP
Before building, open `sip_defs.rs` and set `SIP_SERVER_IP_ADDRESS` to your server’s actual runtime IP:

pub const SIP_SERVER_IP_ADDRESS: &str = "192.168.184.128"; // Example—change as needed


### NOTE 2: SIP Numbers Available for Registration
The array below lists the only SIP phone numbers that can register. To add or change users, simply edit the username entries (e.g., “1001”–“1006”). Do not touch the default ip_str and port values — these will be automatically overwritten with each phone’s actual IP and port upon REGISTER.

lazy_static! {

    pub static ref LOCATION_ENTRIES: Mutex<Vec<LocationEntry>> = Mutex::new(vec![
    
        LocationEntry { username: "1001".to_string(), password: "defaultpassword".to_string(), ip_str: "192.168.192.1".to_string(), port: 5060, realm: SIP_SERVER_IP_ADDRESS.to_string(), registered: false, current_addr: None },
        
        LocationEntry { username: "1002".to_string(), password: "defaultpassword".to_string(), ip_str: "192.168.192.1".to_string(), port: 5070, realm: SIP_SERVER_IP_ADDRESS.to_string(), registered: false, current_addr: None },
        
        LocationEntry { username: "1003".to_string(), password: "defaultpassword".to_string(), ip_str: "192.168.1.103".to_string(), port: 5060, realm: SIP_SERVER_IP_ADDRESS.to_string(), registered: false, current_addr: None },
        
        LocationEntry { username: "1004".to_string(), password: "defaultpassword".to_string(), ip_str: "192.168.1.104".to_string(), port: 5060, realm: SIP_SERVER_IP_ADDRESS.to_string(), registered: false, current_addr: None },
        
        LocationEntry { username: "1005".to_string(), password: "defaultpassword".to_string(), ip_str: "192.168.184.1".to_string(), port: 5060, realm: SIP_SERVER_IP_ADDRESS.to_string(), registered: false, current_addr: None },
        
        LocationEntry { username: "1006".to_string(), password: "defaultpassword".to_string(), ip_str: "192.168.184.1".to_string(), port: 5070, realm: SIP_SERVER_IP_ADDRESS.to_string(), registered: false, current_addr: None },
        
        // Add more users as needed
        
    ]);
    
}

For a deeper dive into the state machine, see:

State Machine Design.pdf
