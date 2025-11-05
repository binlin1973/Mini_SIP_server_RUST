mod common;

use common::dummy_socket_addr;
use sip_server_rust::network_utils::send_sip_message;
use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn send_sip_message_returns_ok_for_localhost() {
    let socket = match UdpSocket::bind("127.0.0.1:0") {
        Ok(sock) => sock,
        Err(err) => {
            eprintln!("Skipping send_sip_message test: {err}");
            return;
        }
    };
    socket
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();
    let socket = Arc::new(socket);

    let msg = b"OPTIONS sip:dummy SIP/2.0\r\n\r\n";
    // There may be no listener, but send_to should still succeed locally.
    send_sip_message(&socket, msg, &dummy_socket_addr());
}
