mod common;

use common::sample_invite;
use sip_server_rust::sip_defs::{CallMap, CallState, SipMessage, LOCATION_ENTRIES};
use sip_server_rust::worker::process_sip_messages;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

fn make_sip_message(body: &str, addr: SocketAddr) -> SipMessage {
    SipMessage {
        buffer: body.as_bytes().to_vec(),
        client_addr: addr,
    }
}

#[test]
fn simulate_basic_call_flow() {
    let call_map = Arc::new(Mutex::new(CallMap::new()));
    let (tx, rx) = mpsc::channel();
    let socket = match UdpSocket::bind("127.0.0.1:0") {
        Ok(sock) => Arc::new(sock),
        Err(err) => {
            eprintln!("Skipping integration test; unable to bind UDP socket: {err}");
            return;
        }
    };
    socket
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();

    let call_map_clone = Arc::clone(&call_map);
    let socket_clone = Arc::clone(&socket);
    let handle = thread::spawn(move || process_sip_messages(rx, call_map_clone, socket_clone));

    let inviter: SocketAddr = "127.0.0.1:6000".parse().unwrap();
    let callee_addr: SocketAddr = "127.0.0.1:7000".parse().unwrap();

    {
        let mut entries = LOCATION_ENTRIES.lock().unwrap();
        if let Some(entry) = entries.iter_mut().find(|e| e.username == "1002") {
            entry.current_addr = Some(callee_addr);
            entry.registered = true;
        }
    }

    let invite = sample_invite();
    tx.send(make_sip_message(&invite, inviter)).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(100));
    let b_call_id = {
        let guard = call_map.lock().unwrap();
        let call = &guard.calls[0];
        assert!(call.is_active, "call should be active after INVITE");
        call.b_leg_uuid.clone()
    };

    let ringing = format!(
        "SIP/2.0 180 Ringing\r\n\
Via: SIP/2.0/UDP 192.168.1.20:5060;branch=z9hG4bK776asdhds\r\n\
From: \"Bob\" <sip:1002@server>;tag=asdf\r\n\
To: \"Alice\" <sip:1001@server>;tag=1928301774\r\n\
Call-ID: {}\r\n\
CSeq: 314159 INVITE\r\n\
Contact: <sip:1002@192.168.1.20:5060>\r\n\
Content-Length: 0\r\n\r\n",
        b_call_id
    );
    tx.send(make_sip_message(&ringing, callee_addr)).unwrap();

    let ok = format!(
        "SIP/2.0 200 OK\r\n\
Via: SIP/2.0/UDP 192.168.1.20:5060;branch=z9hG4bK776asdhds\r\n\
From: \"Bob\" <sip:1002@server>;tag=asdf\r\n\
To: \"Alice\" <sip:1001@server>;tag=1928301774\r\n\
Contact: <sip:1002@192.168.1.20:5060>\r\n\
Call-ID: {}\r\n\
CSeq: 314159 INVITE\r\n\
Content-Length: 0\r\n\r\n",
        b_call_id
    );
    tx.send(make_sip_message(&ok, callee_addr)).unwrap();

    let ack = "ACK sip:1002@server SIP/2.0\r\nCall-ID: a84b4c76e66710@pc33.atlanta.com\r\nCSeq: 314159 ACK\r\n\r\n";
    tx.send(make_sip_message(ack, inviter)).unwrap();

    let bye = "BYE sip:1002@server SIP/2.0\r\nVia: SIP/2.0/UDP 192.168.1.10:5060;branch=z9hG4bKbyeA\r\nFrom: \"Alice\" <sip:1001@server>;tag=1928301774\r\nTo: \"Bob\" <sip:1002@server>;tag=asdf\r\nCall-ID: a84b4c76e66710@pc33.atlanta.com\r\nCSeq: 314160 BYE\r\nContent-Length: 0\r\n\r\n";
    tx.send(make_sip_message(bye, inviter)).unwrap();

    let bye_ok = format!(
        "SIP/2.0 200 OK\r\n\
Via: SIP/2.0/UDP 192.168.1.20:5060;branch=z9hG4bKbyeB\r\n\
From: \"Bob\" <sip:1002@server>;tag=asdf\r\n\
To: \"Alice\" <sip:1001@server>;tag=1928301774\r\n\
Call-ID: {}\r\n\
CSeq: 314160 BYE\r\n\
Content-Length: 0\r\n\r\n",
        b_call_id
    );
    tx.send(make_sip_message(&bye_ok, callee_addr)).unwrap();

    thread::sleep(Duration::from_millis(100));
    drop(tx);
    thread::sleep(Duration::from_millis(200));
    handle.join().unwrap();

    let guard = call_map.lock().unwrap();
    assert_eq!(guard.size, 0);
    let call = &guard.calls[0];
    assert!(call.a_leg_uuid.is_empty());
    assert_eq!(call.call_state, CallState::Idle);
}
