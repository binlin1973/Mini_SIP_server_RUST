use crate::sip_defs::*;

// Function to extract CSeq number from CSeq header string
pub fn extract_cseq_number(cseq_header: &str) -> Option<u32> {
    // Example: "CSeq: 1 INVITE"
    cseq_header
        .split_whitespace()
        .nth(1) // Get the second part (the number)
        .and_then(|num_str| num_str.parse::<u32>().ok()) // Try to parse it as u32
}

// Extracts the method or status code from the first line
pub fn parse_first_line(first_line: &str) -> Option<(i32, String)> {
    let parts: Vec<&str> = first_line.splitn(3, ' ').collect();
    if parts.len() >= 2 {
        if parts[0].starts_with("SIP/2.0") {
            // Response: SIP/2.0 200 OK
            if let Ok(_code) = parts[1].parse::<u32>() {
                return Some((STATUS_CODE, parts[1].to_string()));
            }
        } else {
            // Request: INVITE sip:user@host SIP/2.0
             // Check if it's a known method (case-insensitive check might be better)
            let method = parts[0].to_uppercase();
             match method.as_str() {
                 "INVITE" | "ACK" | "BYE" | "CANCEL" | "REGISTER" | "OPTIONS" => { // Add other methods if needed
                    return Some((REQUEST_METHOD, parts[0].to_string())); // Return original case
                 }
                 _ => return None, // Unknown method
            }
        }
    }
    None
}

// Extracts a specific header's value
// Example: get_header_value("Via: SIP/2.0/UDP ...\r\n", "Via:") -> " SIP/2.0/UDP ..."
pub fn get_header_value<'a>(message_str: &'a str, header_name: &str) -> Option<&'a str> {
    if let Some(start_pos) = message_str.find(header_name) {
        let header_start = start_pos + header_name.len();
        if let Some(end_pos) = message_str[header_start..].find("\r\n") {
            // Skip leading whitespace after ':'
            let value_start = message_str[header_start..header_start + end_pos]
                              .find(|c: char| !c.is_whitespace())
                              .map(|i| header_start + i)
                              .unwrap_or(header_start); // handle case with no value after ':'
            return Some(&message_str[value_start..header_start + end_pos]);
        }
    }
    None
}
// Extracts Call-ID
pub fn get_call_id(message_str: &str) -> Option<String> {
    get_header_value(message_str, "Call-ID:")
        .map(|s| s.trim().to_string())
}

// Extracts From header value
pub fn get_from_header(message_str: &str) -> Option<String> {
     get_header_value(message_str, "From:")
        .map(|s| format!("From: {}",s)) // Reconstruct full header for storage
}
// Extracts To header value
pub fn get_to_header(message_str: &str) -> Option<String> {
    get_header_value(message_str, "To:")
        .map(|s| format!("To: {}",s))
}
// Extracts Via header value (first Via)
pub fn get_via_header(message_str: &str) -> Option<String> {
    get_header_value(message_str, "Via:")
       .map(|s| format!("Via: {}",s))
}
// Extracts CSeq header value
pub fn get_cseq_header(message_str: &str) -> Option<String> {
     get_header_value(message_str, "CSeq:")
        .map(|s| format!("CSeq: {}",s))
}

// Extracts Contact header value
pub fn get_contact_header(message_str: &str) -> Option<String> {
     get_header_value(message_str, "Contact:")
        .map(|s| format!("Contact: {}",s))
}

// Extracts Max-Forwards header value
pub fn get_max_forwards(message_str: &str) -> Option<u32> {
     get_header_value(message_str, "Max-Forwards:")
         .and_then(|s| s.trim().parse::<u32>().ok())
}


// Extracts SDP (checks Content-Type and returns content after blank line)
pub fn get_sdp_body(message_str: &str) -> Option<&str> {
    if get_header_value(message_str, "Content-Type:")
        .map_or(false, |ct| ct.trim().contains("application/sdp"))
    {
        message_str.split_once("\r\n\r\n").map(|(_, body)| body)
    } else {
        None
    }
}


// Extracts username from From or To header URI (e.g., "sip:1001@host" -> "1001")
pub fn extract_username_from_uri(uri_header: &str) -> Option<String> {
    // Find <sip:...> or <tel:...>
    if let Some(uri_start) = uri_header.find('<') {
        if let Some(uri_end) = uri_header[uri_start..].find('>') {
            let uri = &uri_header[uri_start + 1 .. uri_start + uri_end];
            // Find "sip:" or "tel:" prefix
            let user_part_start = uri.find(':').map(|i| i + 1).unwrap_or(0);
            // Find '@'
            let user_part_end = uri[user_part_start..].find('@').map(|i| user_part_start + i).unwrap_or(uri.len());

            let username = &uri[user_part_start..user_part_end];
             if !username.is_empty() && username.len() < MAX_USERNAME_LENGTH {
                 return Some(username.to_string());
            }
        }
    }
    None
}

// Extracts host and port from Via header
pub fn extract_via_host_port(via_header: &str) -> Option<(String, u16)> {
    // Via: SIP/2.0/UDP host:port;branch=...
    if let Some(host_start) = via_header.find(|c: char| c.is_alphanumeric() || c == '.' || c == '-') {
        if let Some(host_end) = via_header[host_start..].find(|c: char| !c.is_alphanumeric() && c != '.' && c != '-') {
            let host_part = &via_header[host_start..host_start + host_end];
            let parts: Vec<&str> = host_part.split(':').collect();
            let host = parts.get(0)?.to_string();
            let port = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(SIP_PORT); // Default port
             return Some((host, port));
        }
    }
    None
}

// Extracts received IP and rport from Via header parameters
pub fn extract_via_received_rport(via_header: &str) -> (Option<String>, Option<u16>) {
    let mut received = None;
    let mut rport = None;
    for param in via_header.split(';') {
        if let Some(rec) = param.trim().strip_prefix("received=") {
            received = Some(rec.to_string());
        } else if let Some(rp) = param.trim().strip_prefix("rport=") {
             rport = rp.parse().ok();
         } else if param.trim() == "rport" { // Handle flag rport
             rport = Some(0); // Indicate rport flag is present
        }
    }
    (received, rport)
}