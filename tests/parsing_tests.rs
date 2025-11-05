mod common;

use common::{malformed_message, sample_invite};
use sip_server_rust::parsing::*;
use sip_server_rust::sip_defs::REQUEST_METHOD;

#[test]
fn parse_basic_headers() {
    let invite = sample_invite();
    assert_eq!(
        parse_first_line(invite.lines().next().unwrap()),
        Some((REQUEST_METHOD, "INVITE".into()))
    );
    assert!(get_via_header(&invite)
        .unwrap()
        .contains("branch=z9hG4bK776asdhds"));
    assert!(get_from_header(&invite).unwrap().contains("Alice"));
    assert!(get_to_header(&invite).unwrap().contains("Bob"));
    assert!(get_contact_header(&invite)
        .unwrap()
        .contains("1001@192.168.1.10:5060"));
    assert_eq!(
        get_call_id(&invite).unwrap(),
        "a84b4c76e66710@pc33.atlanta.com"
    );
    assert_eq!(get_cseq_header(&invite).unwrap(), "CSeq: 314159 INVITE");
}

#[test]
fn handle_malformed_without_panics() {
    let msg = malformed_message();

    assert!(parse_first_line(msg.lines().next().unwrap()).is_some());
    assert!(get_via_header(&msg).is_none());
    assert!(get_from_header(&msg).is_none());
    assert!(get_contact_header(&msg).is_none());
    assert_eq!(get_max_forwards(&msg), None);
    assert!(extract_username_from_uri(&msg).is_none());
}

#[test]
fn content_type_detection_respects_case() {
    let mut invite = sample_invite();
    invite.push_str("Content-Type: application/sdp\r\n\r\nv=0\r\n");
    assert!(get_sdp_body(&invite).is_some());

    let mut no_sdp = sample_invite();
    no_sdp.push_str("Content-Type: text/plain\r\n\r\nhello");
    assert!(get_sdp_body(&no_sdp).is_none());
}
