# Mini_SIP_server_RUST
writen by RUST, the SIP server to handle standard SIP call flows, focusing on managing call setup and teardown; states from the initial INVITE to the final 200 OK response to the BYE request, including interim states.

# !!!NOTE: Set this to your server's actual IP address in sip_defs.rs before build !!!

pub const SIP_SERVER_IP_ADDRESS: &str = "192.168.184.128"; // Example, change as needed
