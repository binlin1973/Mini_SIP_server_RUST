mod common;

use common::sample_invite;
use sip_server_rust::parsing::*;

fn invite_without(header: &str) -> String {
    let mut result = String::new();
    for segment in sample_invite().split_inclusive("\r\n") {
        if !segment.starts_with(header) {
            result.push_str(segment);
        }
    }
    result
}

#[test]
fn missing_headers_return_none() {
    let cases = [
        ("Via:", get_via_header as fn(&str) -> Option<String>),
        ("From:", get_from_header),
        ("To:", get_to_header),
        ("Contact:", get_contact_header),
        ("CSeq:", get_cseq_header),
        ("Call-ID:", |msg| get_call_id(msg).map(|id| format!("Call-ID: {id}"))),
    ];

    for (header, getter) in cases {
        let msg = invite_without(header);
        assert!(
            getter(&msg).is_none(),
            "removing {header} should cause helper to return None"
        );
    }
}

#[test]
fn duplicate_conflicting_headers_choose_first_match() {
    let mut msg = sample_invite();
    msg.push_str("Call-ID: second@conflict\r\n");
    msg.push_str("Via: SIP/2.0/UDP 10.0.0.1;branch=dup\r\n\r\n");

    let call_id = get_call_id(&msg).expect("first Call-ID should be parsed");
    assert_eq!(
        call_id, "a84b4c76e66710@pc33.atlanta.com",
        "first Call-ID should take precedence"
    );

    let via = get_via_header(&msg).expect("first Via should be parsed");
    assert!(
        via.contains("192.168.1.10"),
        "helper should not skip the initial Via header"
    );
}

#[test]
fn header_name_casing_is_case_sensitive() {
    let lower = sample_invite().replace("Via:", "via:");
    assert!(
        get_via_header(&lower).is_none(),
        "lowercase header name should not be detected"
    );

    let upper = sample_invite().replace("From:", "FROM:");
    assert!(
        get_from_header(&upper).is_none(),
        "uppercase header should not match"
    );

    let mixed = sample_invite().replace("Contact:", "ConTact:");
    assert!(
        get_contact_header(&mixed).is_none(),
        "mixed-case header should not match"
    );
}

#[test]
fn malformed_lines_without_crlf_or_with_trailing_spaces() {
    let no_crlf = "INVITE sip:1002@server SIP/2.0\nVia: broken line without crlf\n";
    assert!(
        parse_first_line(no_crlf.lines().next().unwrap()).is_some(),
        "helper should tolerate line endings without CRLF"
    );
    assert!(
        get_via_header(no_crlf).is_none(),
        "header parser should not misread newline-separated entries"
    );

    let mut spaced = sample_invite();
    spaced = spaced.replace(
        "Call-ID: a84b4c76e66710@pc33.atlanta.com\r\n",
        "Call-ID:  trimmed-id   \r\n",
    );
    let call_id = get_call_id(&spaced).expect("Call-ID should still be found");
    assert_eq!(call_id, "trimmed-id", "extracted value should be trimmed");
}

#[test]
fn cseq_with_invalid_number_or_method_returns_none() {
    let bad_number = sample_invite().replace("CSeq: 314159 INVITE", "CSeq: abc INVITE");
    assert!(
        extract_cseq_number(&bad_number).is_none(),
        "non-numeric CSeq should return None"
    );

    let mut bad_method = sample_invite();
    bad_method = bad_method.replace(
        "INVITE sip:1002@server SIP/2.0",
        "FOO sip:1002@server SIP/2.0",
    );
    assert!(
        parse_first_line(bad_method.lines().next().unwrap()).is_none(),
        "unknown request method should be rejected"
    );
}

#[test]
fn contact_without_matching_angle_brackets() {
    let missing = sample_invite().replace(
        "<sip:1001@192.168.1.10:5060>",
        "sip:1001@192.168.1.10:5060",
    );
    let contact = get_contact_header(&missing).expect("header should still exist");
    assert!(
        extract_username_from_uri(&contact).is_none(),
        "URI extractor should fail without brackets"
    );

    let mismatched = sample_invite().replace(
        "<sip:1001@192.168.1.10:5060>",
        "<sip:1001@192.168.1.10:5060",
    );
    let contact = get_contact_header(&mismatched).expect("header present but broken");
    assert!(
        extract_username_from_uri(&contact).is_none(),
        "missing closing bracket should prevent parsing"
    );
}

#[test]
fn very_long_headers_do_not_panic_and_are_trimmed() {
    let long_token = "A".repeat(2048);
    let mut msg = sample_invite();
    msg.push_str(&format!("X-Custom: {}\r\n", long_token));
    msg.push_str(&format!("Contact: <sip:{}@example.com>   \r\n", long_token));

    assert!(
        parse_first_line(msg.lines().next().unwrap()).is_some(),
        "helper should still parse a valid first line"
    );

    let contact = get_contact_header(&msg).expect("contact should be present");
    assert!(
        contact.ends_with('>'),
        "contact helper should trim trailing spaces"
    );
}
