# Mini_SIP_server_RUST
writen by RUST, the SIP server to handle standard SIP call flows, focusing on managing call setup and teardown; states from the initial INVITE to the final 200 OK response to the BYE request, including interim states.

# NOTE: Before building, open sip_defs.rs and set SIP_SERVER_IP_ADDRESS to the actual IP address your server will use at runtime:
pub const SIP_SERVER_IP_ADDRESS: &str = "192.168.184.128"; // Example—change as needed
