use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[allow(dead_code)]
pub fn sample_invite() -> String {
    "INVITE sip:1002@server SIP/2.0\r\n\
Via: SIP/2.0/UDP 192.168.1.10:5060;branch=z9hG4bK776asdhds\r\n\
Max-Forwards: 70\r\n\
From: \"Alice\" <sip:1001@server>;tag=1928301774\r\n\
To: \"Bob\" <sip:1002@server>\r\n\
Contact: <sip:1001@192.168.1.10:5060>\r\n\
Call-ID: a84b4c76e66710@pc33.atlanta.com\r\n\
CSeq: 314159 INVITE\r\n\
Content-Length: 0\r\n\r\n"
        .to_string()
}

#[allow(dead_code)]
pub fn sample_response(status: u16, reason: &str) -> String {
    format!(
        "SIP/2.0 {} {}\r\n\
Via: SIP/2.0/UDP 192.168.1.20:5060;branch=z9hG4bK776asdhds\r\n\
From: \"Bob\" <sip:1002@server>;tag=asdf\r\n\
To: \"Alice\" <sip:1001@server>;tag=qwer\r\n\
Contact: <sip:1002@192.168.1.20:5060>\r\n\
Call-ID: a84b4c76e66710@pc33.atlanta.com\r\n\
CSeq: 314159 INVITE\r\n\
Content-Length: 0\r\n\r\n",
        status, reason
    )
}

#[allow(dead_code)]
pub fn dummy_socket_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5060)
}

#[allow(dead_code)]
pub fn malformed_message() -> String {
    "INVITE sip:missing-headers SIP/2.0\r\n\
CSeq: 1 INVITE\r\n\
Content-Length: 0\r\n\r\n"
        .to_string()
}
