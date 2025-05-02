# Mini_SIP_server_RUST
writen by RUST, the SIP server to handle standard SIP call flows, focusing on managing call setup and teardown; states from the initial INVITE to the final 200 OK response to the BYE request, including interim states.

// NOTE: Before building, open sip_defs.rs and set SIP_SERVER_IP_ADDRESS to the actual IP address your server will use at runtime:
pub const SIP_SERVER_IP_ADDRESS: &str = "192.168.184.128"; // Example—change as needed


// NOTE: The following entries define the actual SIP phone numbers that can register.

// You only need to set or add SIP phone numbers (e.g., “1001”–“1006”). 
// Do NOT modify the `ip_str` and `port` fields—those defaults will be automatically replaced with each phone’s actual IP and port upon REGISTER.

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
