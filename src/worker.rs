use crate::sip_defs::*;
use crate::sip_defs::CallMap; // Correct import path
use crate::network_utils::send_sip_message;
use crate::parsing::*; // Import parsing helpers
use std::sync::{mpsc::Receiver, Arc, Mutex};
use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};


// Main function for worker threads
pub fn process_sip_messages(
    receiver: Receiver<SipMessage>, // Each worker gets its own receiver end
    call_map: Arc<Mutex<CallMap>>,
    socket: Arc<UdpSocket>, // Shared socket for sending
) {
    println!("Worker thread started.");
    loop {
        match receiver.recv() { // Blocks until a message is received
            Ok(message) => {
                let source_addr = message.client_addr;
                let message_str = String::from_utf8_lossy(&message.buffer); // For parsing

                println!("\n================ RX from {} ================\n{}\n================================================",
                    source_addr, message_str);

                // Basic Parsing
                let first_line = message_str.lines().next().unwrap_or("");
                let call_id = get_call_id(&message_str).unwrap_or_default();
                let has_sdp = get_sdp_body(&message_str).is_some();

                // Determine message type (Request/Response) and method/code
                if let Some((msg_type, method_or_code)) = parse_first_line(first_line) {

                    // Handle REGISTER separately (doesn't use CallMap in the same way)
                    if msg_type == REQUEST_METHOD && method_or_code == "REGISTER" {
                        handle_register(&message, &socket, &message_str);
                    } else {
                         // Lock CallMap for find/allocate/update operations
                         let mut map_guard = call_map.lock().expect("Failed to lock CallMap");

                        // Find existing call or allocate new one for INVITE
                        let (call_opt, leg_type) = CallMap::find_call_by_callid_mut(&mut map_guard, &call_id);

                        if let Some(call) = call_opt {
                            // Existing call found
                            handle_state_machine(
                                call,
                                msg_type,
                                &method_or_code,
                                has_sdp,
                                &message, // Pass original SipMessage with SocketAddr
                                &message_str,
                                leg_type,
                                &socket
                            );
                        } else if msg_type == REQUEST_METHOD && method_or_code == "INVITE" {
                             // No existing call, but it's an INVITE - try to allocate
                             println!("  Call-ID [{}] not found, processing INVITE to allocate.", call_id);
                             // Allocate returns index now
                             if let Some(new_call_index) = CallMap::allocate_new_call_mut(&mut map_guard) {
                                println!("  Allocated new call at index {}", new_call_index);
                                // Get mutable reference using the index *after* allocation
                                let new_call = &mut map_guard.calls[new_call_index];
                                handle_state_machine(
                                    new_call, // Pass the mutable ref to the new call
                                    msg_type,
                                    &method_or_code,
                                    has_sdp,
                                    &message,
                                    &message_str,
                                    A_LEG, // Initial INVITE is always A_LEG perspective
                                    &socket
                                );
                            } else {
                                eprintln!("Error: CallMap full, cannot allocate for INVITE Call-ID [{}]", call_id);
                                // Send 503 Service Unavailable?
                                // Need headers from the INVITE to respond properly
                                if let (Some(via), Some(from), Some(to), Some(cseq)) = (
                                    get_via_header(&message_str),
                                    get_from_header(&message_str),
                                    get_to_header(&message_str),
                                    get_cseq_header(&message_str))
                                {
                                    // Corrected format! usage
                                    let response_503 = format!(
                                        "SIP/2.0 503 Service Unavailable\r\n\
                                        {}\r\n\
                                        {}\r\n\
                                        {}\r\n\
                                        Call-ID: {}\r\n\
                                        {}\r\n\
                                        User-Agent: TinySIP-Rust\r\n\
                                        Content-Length: 0\r\n\r\n",
                                        via,
                                        from,
                                        to,
                                        call_id, // Use the extracted call_id here
                                        cseq
                                    );
                                    send_sip_message(&socket, response_503.as_bytes(), &source_addr);
                                }
                            }
                        } else {
                            // Message for a non-existent call, and not an INVITE
                            println!("  Ignoring message for non-existent Call-ID [{}], Method/Code [{}], Type [{}]", call_id, method_or_code, msg_type);
                            // Maybe send a 481 Call/Transaction Does Not Exist response? Requires CSeq etc.
                        }
                        // MutexGuard is dropped here, releasing the lock
                    }

                } else {
                    eprintln!("Failed to parse first line: {}", first_line);
                }
            }
            Err(e) => {
                eprintln!("Worker thread receive error: {}. Stopping.", e);
                break; // Exit loop if channel disconnects
            }
        }
    }
    println!("Worker thread finished.");
}


// --- REGISTER Handling ---
fn handle_register(
    message: &SipMessage,
    socket: &Arc<UdpSocket>,
    message_str: &str,
) {
    println!("Handling REGISTER request.");

    // Extract necessary headers for response
    let via = get_via_header(message_str).unwrap_or_default();
    let from = get_from_header(message_str).unwrap_or_default();
    let to = get_to_header(message_str).unwrap_or_default();
    let call_id = get_call_id(message_str).unwrap_or_default();
    let cseq = get_cseq_header(message_str).unwrap_or_default();
    let contact = get_contact_header(message_str).unwrap_or_default(); // Get Contact for 200 OK

    // Extract username from From header
    let username = extract_username_from_uri(&from);

    if let Some(uname) = username {
        // Update the location entry with the source address of the REGISTER request
        if update_location_entry_addr(&uname, message.client_addr) {
            // User found and updated, send 200 OK
            // Add Expires header to Contact if needed (parsing original helps)
             let contact_with_expires = if contact.is_empty() { "".to_string() } else { format!("{};expires=7200", contact) }; // Simplified

            // Corrected format! usage
            let response_200 = format!(
                "SIP/2.0 200 OK\r\n\
                {}\r\n\
                {}\r\n\
                {}\r\n\
                Call-ID: {}\r\n\
                {}\r\n\
                {}\r\n\
                User-Agent: TinySIP-Rust\r\n\
                Content-Length: 0\r\n\r\n",
                via,
                from,
                to,
                call_id,
                cseq,
                contact_with_expires
            );
            println!("REGISTER successful for {}. Sending 200 OK.", uname);
            send_sip_message(socket, response_200.as_bytes(), &message.client_addr);

        } else {
            // User not found in static list
            // Corrected format! usage
            let response_404 = format!(
                "SIP/2.0 404 Not Found\r\n\
                {}\r\n\
                {}\r\n\
                {}\r\n\
                Call-ID: {}\r\n\
                {}\r\n\
                User-Agent: TinySIP-Rust\r\n\
                Content-Length: 0\r\n\r\n",
                via,
                from,
                to,
                call_id,
                cseq
            );
            println!("User '{}' not found. Sending 404 Not Found.", uname);
            send_sip_message(socket, response_404.as_bytes(), &message.client_addr);
        }
    } else {
        eprintln!("Failed to extract username from From header: {}", from);
        // Optionally send a 400 Bad Request
    }
}


// --- State Machine ---
// Note: Takes a mutable reference to the call, assuming the CallMap lock is held
#[allow(clippy::too_many_arguments)] // Match C function signature
#[allow(clippy::cognitive_complexity)] // State machine is complex
fn handle_state_machine(
    call: &mut Call, // Takes mutable reference, lock is held outside
    message_type: i32,
    method_or_code: &str,
    has_sdp: bool,
    message: &SipMessage, // Contains client_addr
    raw_sip_message: &str,
    leg_type: i32,
    socket: &Arc<UdpSocket>,
) {
    println!(
        "  State Machine: Call Index [{}], State [{:?}], RX Msg Type [{}], Code/Method [{}], Leg [{}]",
        call.index, call.call_state, message_type, method_or_code, if leg_type == A_LEG {"A"} else {"B"}
    );

    // Extract common headers from incoming message for potential use in responses
    let via_header = get_via_header(raw_sip_message).unwrap_or_default();
    let from_header = get_from_header(raw_sip_message).unwrap_or_default();
    let to_header = get_to_header(raw_sip_message).unwrap_or_default();
    let call_id_header = get_call_id(raw_sip_message).unwrap_or_default(); // Already have call.a/b_leg_uuid
    let cseq_header = get_cseq_header(raw_sip_message).unwrap_or_default();
    let contact_header = get_contact_header(raw_sip_message).unwrap_or_default(); // Contact from incoming message
    let max_forwards = get_max_forwards(raw_sip_message).unwrap_or(70);


    // --- State Machine Logic ---
    // This closely follows the C logic, adapted for Rust types and helpers.
    // Locking Note: The `call` is already mutable, implying the lock is held.

     // Refresh To header for B leg if message came from B leg (needed for ACK/BYE construction)
     if leg_type == B_LEG && !to_header.is_empty() {
        call.b_leg_header.to = to_header.clone();
     }
     // Refresh To header for A leg if message came from A leg
      if leg_type == A_LEG && !to_header.is_empty() {
         call.a_leg_header.to = to_header.clone();
     }


    match call.call_state {
        CallState::Idle => {
            // Should only happen for the initial INVITE, which is handled before this match
             // by the allocation logic calling handle_state_machine the first time.
             if message_type == REQUEST_METHOD && method_or_code == "INVITE" && leg_type == A_LEG {
                 println!("  Processing initial INVITE for allocated call {}", call.index);

                // 1. Store A-leg info
                call.a_leg_addr = Some(message.client_addr);
                call.a_leg_uuid = call_id_header.clone();
                // Create unique B-leg ID - Ensure it fits within MAX_UUID_LENGTH
                let base_id = if call_id_header.len() > 6 { &call_id_header[6..] } else { &call_id_header };
                call.b_leg_uuid = format!("b-leg-{}", base_id);
                if call.b_leg_uuid.len() >= MAX_UUID_LENGTH {
                    call.b_leg_uuid.truncate(MAX_UUID_LENGTH - 1);
                }

                // Store A-leg headers
                // Update Via with received/rport before storing
                let (via_received, via_rport) = extract_via_received_rport(&via_header);
                let mut updated_via = via_header.clone();
                 if via_rport.is_some() || via_received.is_none() { // Add received if rport present or received missing
                    if !via_header.contains(";received=") {
                         updated_via = updated_via.trim_end_matches("\r\n").to_string(); // Remove existing CRLF
                         updated_via.push_str(&format!(";received={}", message.client_addr.ip()));
                         updated_via.push_str("\r\n"); // Add CRLF back
                    }
                 }
                 if via_rport == Some(0) { // rport flag without value
                     if !via_header.contains(";rport=") {
                        updated_via = updated_via.trim_end_matches("\r\n").to_string();
                        updated_via.push_str(&format!(";rport={}", message.client_addr.port()));
                        updated_via.push_str("\r\n");
                     }
                 }
                 call.a_leg_header.via = updated_via;
                 call.a_leg_header.from = from_header.clone();
                 call.a_leg_header.to = to_header.clone();
                 call.a_leg_header.cseq = cseq_header.clone();

                 // Extract and store A-leg Contact URI
                 if let Some(start_idx) = contact_header.find('<') {
                     if let Some(end_idx) = contact_header.find('>') {
                         if start_idx < end_idx {
                             call.a_leg_contact = contact_header[start_idx + 1..end_idx].to_string();
                         }
                     }
                 } else {
                    // Maybe extract from "Contact: sip:user@host:port" format
                     if let Some(stripped) = contact_header.strip_prefix("Contact:").map(|s| s.trim()) {
                         call.a_leg_contact = stripped.to_string();
                     }
                 }

                 if has_sdp {
                     call.a_leg_media.remote_media = true; // A-leg received remote SDP (from its perspective)
                     call.b_leg_media.local_media = true; // B-leg will send local SDP (based on A's offer)
                 }

                 // 2. Find Callee (B-leg) address
                 if let Some(callee_username) = extract_username_from_uri(&to_header) {
                     call.callee = callee_username.clone(); // Store callee username
                    if let Some(callee_addr) = get_registered_addr(&callee_username) {
                        call.b_leg_addr = Some(callee_addr);
                         println!("  Found registered location for callee '{}': {}", callee_username, callee_addr);

                         // 3. Send 100 Trying to A-leg
                         // Corrected format! usage
                         let trying_100 = format!(
                            "SIP/2.0 100 Trying\r\n\
                            {}\
                            {}\r\n\
                            {}\r\n\
                            Call-ID: {}\r\n\
                            {}\r\n\
                            User-Agent: TinySIP-Rust\r\n\
                            Content-Length: 0\r\n\r\n",
                            call.a_leg_header.via,
                            call.a_leg_header.from,
                            call.a_leg_header.to,
                            call.a_leg_uuid,
                            call.a_leg_header.cseq
                         );
                         send_sip_message(socket, trying_100.as_bytes(), &call.a_leg_addr.unwrap());

                         // 4. Prepare and send INVITE to B-leg
                         let b_branch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                         let b_via = format!("Via: SIP/2.0/UDP {}:{};branch=z9hG4bK{}", SIP_SERVER_IP_ADDRESS, SIP_PORT, b_branch);
                         let b_cseq_num = next_cseq();
                         let b_cseq = format!("CSeq: {} INVITE", b_cseq_num);
                         let b_contact = format!("Contact: <sip:TinySIP@{}:{}>", SIP_SERVER_IP_ADDRESS, SIP_PORT);
                         let b_to = format!("To: <sip:{}@{}>", call.callee, callee_addr.ip()); // Simple To for B

                         // Store B-leg headers we generate
                          call.b_leg_header.via = format!("{}{}",b_via,"\r\n");
                          call.b_leg_header.from = call.a_leg_header.from.clone(); // Use A's From
                          call.b_leg_header.to = format!("{}{}",b_to,"\r\n");
                          call.b_leg_header.cseq = format!("{}{}",b_cseq,"\r\n");


                         let sdp_body = get_sdp_body(raw_sip_message).unwrap_or("");
                         let content_length = sdp_body.len();

                         // Corrected format! usage
                         let invite_to_b = format!(
                            "INVITE sip:{}@{} SIP/2.0\r\n\
                            {}\r\n\
                            {}\r\n\
                            {}\r\n\
                            Call-ID: {}\r\n\
                            {}\r\n\
                            Max-Forwards: {}\r\n\
                            {}\r\n\
                            User-Agent: TinySIP-Rust\r\n\
                            Content-Type: application/sdp\r\n\
                            Content-Length: {}\r\n\r\n\
                            {}",
                            call.callee, callee_addr, // Request-URI target
                            b_via, // B-leg Via
                            call.a_leg_header.from, // A-leg From
                            b_to, // B-leg To
                            call.b_leg_uuid, // B-leg Call-ID
                            b_cseq, // B-leg CSeq
                            max_forwards.saturating_sub(1),
                            b_contact, // Server Contact
                            content_length,
                            sdp_body
                         );
                         send_sip_message(socket, invite_to_b.as_bytes(), &callee_addr);


                         // 5. Update State
                         call.call_state = CallState::Routing;
                         println!("  Call {} state transitioned to ROUTING.", call.index);

                    } else {
                        println!("  Callee '{}' not found or not registered.", callee_username);
                        // Send 404 Not Found to A-leg
                        // Corrected format! usage
                        let response_404 = format!(
                            "SIP/2.0 404 Not Found\r\n\
                            {}\
                            {}\r\n\
                            {}\r\n\
                            Call-ID: {}\r\n\
                            {}\r\n\
                            User-Agent: TinySIP-Rust\r\n\
                            Content-Length: 0\r\n\r\n",
                            call.a_leg_header.via,
                            call.a_leg_header.from,
                            call.a_leg_header.to,
                            call.a_leg_uuid,
                            call.a_leg_header.cseq
                        );
                         send_sip_message(socket, response_404.as_bytes(), &call.a_leg_addr.unwrap());
                        // Release the allocated call
                         CallMap::init_call(call); // Resets the state including is_active
                         // Size adjustment happens outside if needed, or modify init_call
                        println!("  Call {} released due to callee not found.", call.index);
                    }
                 } else {
                     eprintln!("  Failed to extract callee username from To: {}", to_header);
                     // Send 400 Bad Request?
                     // Release the allocated call
                     CallMap::init_call(call);
                     println!("  Call {} released due to bad To header.", call.index);
                 }
             } else {
                  println!("  Ignoring message type {} code/method {} in IDLE state.", message_type, method_or_code);
             }
        }

        CallState::Routing | CallState::Ringing => {
            if call.call_state == CallState::Routing { println!("  Current State: ROUTING"); }
            if call.call_state == CallState::Ringing { println!("  Current State: RINGING"); }

             match message_type {
                 REQUEST_METHOD => {
                     if method_or_code == "CANCEL" && leg_type == A_LEG {
                         println!("  Processing CANCEL from A leg");
                         // Action 6
                         // 1. Send 200 OK for CANCEL to A leg
                         // Corrected format! usage
                        let ok_200_cancel = format!(
                            "SIP/2.0 200 OK\r\n\
                            {}\r\n\
                            {}\r\n\
                            {}\r\n\
                            Call-ID: {}\r\n\
                            {}\r\n\
                            User-Agent: TinySIP-Rust\r\n\
                            Content-Length: 0\r\n\r\n",
                            via_header, // Via from CANCEL
                            from_header, // From from CANCEL
                            to_header, // To from CANCEL
                            call_id_header, // Call-ID from CANCEL (should match A-leg)
                            cseq_header // CSeq from CANCEL
                         );
                        send_sip_message(socket, ok_200_cancel.as_bytes(), &call.a_leg_addr.unwrap());

                         // 2. Send 487 for original INVITE to A leg
                         // Corrected format! usage
                         let terminated_487 = format!(
                            "SIP/2.0 487 Request Terminated\r\n\
                            {}\
                            {}\r\n\
                            {}\r\n\
                            Call-ID: {}\r\n\
                            {}\r\n\
                            User-Agent: TinySIP-Rust\r\n\
                            Content-Length: 0\r\n\r\n",
                            call.a_leg_header.via, // Stored A-leg Via
                            call.a_leg_header.from, // Stored A-leg From
                            call.a_leg_header.to, // Stored A-leg To
                            call.a_leg_uuid, // Stored A-leg Call-ID
                            call.a_leg_header.cseq // Stored A-leg CSeq
                         );
                        send_sip_message(socket, terminated_487.as_bytes(), &call.a_leg_addr.unwrap());

                         // 3. Send CANCEL to B leg
                         let b_cseq_val = extract_cseq_number(&call.b_leg_header.cseq).unwrap_or(0); // Use B's INVITE CSeq num
                         // Corrected format! usage
                         let cancel_b = format!(
                            "CANCEL sip:{}@{} SIP/2.0\r\n\
                            {}\
                            {}\r\n\
                            {}\r\n\
                            Call-ID: {}\r\n\
                            CSeq: {} CANCEL\r\n\
                            Max-Forwards: 70\r\n\
                            User-Agent: TinySIP-Rust\r\n\
                            Content-Length: 0\r\n\r\n",
                            call.callee, call.b_leg_addr.unwrap(), // B leg target URI
                            call.b_leg_header.via, // Stored B-leg Via
                            call.b_leg_header.from, // Stored B-leg From
                            call.b_leg_header.to, // Stored B-leg To
                            call.b_leg_uuid, // Stored B-leg Call-ID
                            b_cseq_val // Match B's INVITE CSeq num
                         );
                         send_sip_message(socket, cancel_b.as_bytes(), &call.b_leg_addr.unwrap());

                         // 4. Set state to DISCONNECTING
                         call.call_state = CallState::Disconnecting;
                         println!("  Call {} state transitioned to DISCONNECTING.", call.index);

                     } else {
                          println!("  Ignoring METHOD {} from leg {} in state {:?}", method_or_code, leg_type, call.call_state);
                     }
                 }
                 STATUS_CODE => {
                    if leg_type == B_LEG {
                         let code: u16 = method_or_code.parse().unwrap_or(0);
                         match code {
                             100 => { /* Ignore 100 Trying */ }
                             180 => { // Ringing
                                 println!("  Processing 180 Ringing from B leg");
                                 // Action 2
                                 // 1. Forward 180 Ringing to A leg
                                 // Corrected format! usage
                                 let ringing_180_a = format!(
                                    "SIP/2.0 180 Ringing\r\n\
                                    {}\
                                    {}\r\n\
                                    {}\r\n\
                                    Call-ID: {}\r\n\
                                    {}\r\n\
                                    Contact: <sip:TinySIP@{}:{}>\r\n\
                                    User-Agent: TinySIP-Rust\r\n\
                                    {}\r\n", // Placeholder for potential SDP/Content-Length
                                    call.a_leg_header.via,
                                    call.a_leg_header.from,
                                    call.a_leg_header.to,
                                    call.a_leg_uuid,
                                    call.a_leg_header.cseq,
                                    SIP_SERVER_IP_ADDRESS, SIP_PORT,
                                    // Pass through SDP if present in 180? Usually not.
                                    if let Some(sdp) = get_sdp_body(raw_sip_message) {
                                        format!("Content-Type: application/sdp\r\nContent-Length: {}\r\n\r\n{}", sdp.len(), sdp)
                                    } else {
                                         "Content-Length: 0\r\n\r\n".to_string()
                                    }
                                 );
                                 send_sip_message(socket, ringing_180_a.as_bytes(), &call.a_leg_addr.unwrap());

                                 // 2. Update media state if SDP present in 180 (less common)
                                 if has_sdp {
                                     call.a_leg_media.local_media = true; // Server has A's perspective
                                     call.b_leg_media.remote_media = true; // Server has B's perspective
                                 }

                                 // 3. Set state to Ringing
                                 call.call_state = CallState::Ringing;
                                 println!("  Call {} state transitioned to RINGING.", call.index);
                             }
                             183 => { // Session Progress
                                  println!("  Processing 183 Session Progress from B leg");
                                 // Action: Forward 183 to A leg
                                 // Corrected format! usage
                                 let progress_183_a = format!(
                                    "SIP/2.0 183 Session Progress\r\n\
                                    {}\
                                    {}\r\n\
                                    {}\r\n\
                                    Call-ID: {}\r\n\
                                    {}\r\n\
                                    Contact: <sip:TinySIP@{}:{}>\r\n\
                                    User-Agent: TinySIP-Rust\r\n\
                                    {}\r\n", // Placeholder for potential SDP/Content-Length
                                    call.a_leg_header.via,
                                    call.a_leg_header.from,
                                    call.a_leg_header.to,
                                    call.a_leg_uuid,
                                    call.a_leg_header.cseq,
                                    SIP_SERVER_IP_ADDRESS, SIP_PORT,
                                    // Pass through SDP if present in 183
                                    if let Some(sdp) = get_sdp_body(raw_sip_message) {
                                        call.a_leg_media.local_media = true;
                                        call.b_leg_media.remote_media = true;
                                        format!("Content-Type: application/sdp\r\nContent-Length: {}\r\n\r\n{}", sdp.len(), sdp)
                                    } else {
                                         "Content-Length: 0\r\n\r\n".to_string()
                                    }
                                 );
                                 send_sip_message(socket, progress_183_a.as_bytes(), &call.a_leg_addr.unwrap());
                                // State remains Routing or Ringing
                             }
                             200..=299 => { // 2xx Success (typically 200 OK for INVITE)
                                 println!("  Processing 200 OK from B leg");
                                 // Action 3
                                 // Extract B-leg Contact for future use (e.g. Re-INVITE, BYE)
                                 if let Some(start_idx) = contact_header.find('<') {
                                    if let Some(end_idx) = contact_header.find('>') {
                                        if start_idx < end_idx {
                                            call.b_leg_contact = contact_header[start_idx + 1..end_idx].to_string();
                                        }
                                    }
                                 } else {
                                     if let Some(stripped) = contact_header.strip_prefix("Contact:").map(|s| s.trim()){
                                         call.b_leg_contact = stripped.to_string();
                                     }
                                 }
                                 println!("  Extracted B-leg Contact: {}", call.b_leg_contact);

                                 // 1. Forward 200 OK to A leg
                                 // Corrected format! usage
                                 let ok_200_a = format!(
                                    "SIP/2.0 200 OK\r\n\
                                    {}\
                                    {}\r\n\
                                    {}\r\n\
                                    Call-ID: {}\r\n\
                                    {}\r\n\
                                    Contact: <sip:TinySIP@{}:{}>\r\n\
                                    User-Agent: TinySIP-Rust\r\n\
                                    {}\r\n", // Placeholder for potential SDP/Content-Length
                                     call.a_leg_header.via,
                                     call.a_leg_header.from,
                                     call.a_leg_header.to,
                                     call.a_leg_uuid,
                                     call.a_leg_header.cseq,
                                     SIP_SERVER_IP_ADDRESS, SIP_PORT,
                                    // Pass through SDP if present in 200 OK
                                    if let Some(sdp) = get_sdp_body(raw_sip_message) {
                                        call.a_leg_media.local_media = true;
                                        call.b_leg_media.remote_media = true;
                                        format!("Content-Type: application/sdp\r\nContent-Length: {}\r\n\r\n{}", sdp.len(), sdp)
                                    } else {
                                         "Content-Length: 0\r\n\r\n".to_string()
                                    }
                                 );
                                 send_sip_message(socket, ok_200_a.as_bytes(), &call.a_leg_addr.unwrap());

                                // 2. Set state to Answered
                                 call.call_state = CallState::Answered;
                                 println!("  Call {} state transitioned to ANSWERED.", call.index);
                             }
                             300..=699 => { // Failure Response from B leg
                                 println!("  Processing Failure Code {} from B leg", code);
                                 // Action 7
                                // 1. Send ACK to B leg for the failure response
                                 let b_cseq_val = extract_cseq_number(&call.b_leg_header.cseq).unwrap_or(0);
                                 // Corrected format! usage
                                 let b_ack = format!(
                                    "ACK sip:{}@{} SIP/2.0\r\n\
                                    Via: SIP/2.0/UDP {}:{};branch=z9hG4bKack{}\r\n\
                                    {}\r\n\
                                    {}\r\n\
                                    Call-ID: {}\r\n\
                                    CSeq: {} ACK\r\n\
                                    Max-Forwards: 70\r\n\
                                    User-Agent: TinySIP-Rust\r\n\
                                    Content-Length: 0\r\n\r\n",
                                    call.callee, call.b_leg_addr.unwrap(), // B leg target URI
                                    SIP_SERVER_IP_ADDRESS, SIP_PORT, b_cseq_val, // Unique branch for ACK
                                    call.b_leg_header.from, // B leg From
                                    call.b_leg_header.to, // B leg To (potentially updated from response)
                                    call.b_leg_uuid, // B leg Call ID
                                    b_cseq_val // Match B's INVITE CSeq num
                                 );
                                 send_sip_message(socket, b_ack.as_bytes(), &call.b_leg_addr.unwrap());

                                 // 2. Forward the failure response to A leg
                                 // Corrected format! usage
                                 let failure_a = format!(
                                    "SIP/2.0 {}\r\n\
                                    {}\
                                    {}\r\n\
                                    {}\r\n\
                                    Call-ID: {}\r\n\
                                    {}\r\n\
                                    User-Agent: TinySIP-Rust\r\n\
                                    Content-Length: 0\r\n\r\n",
                                    method_or_code, // The failure code and reason phrase from B
                                    call.a_leg_header.via,
                                    call.a_leg_header.from,
                                    call.a_leg_header.to,
                                    call.a_leg_uuid,
                                    call.a_leg_header.cseq
                                 );
                                send_sip_message(socket, failure_a.as_bytes(), &call.a_leg_addr.unwrap());

                                // 3. Set state back to Idle (release call)
                                 CallMap::init_call(call); // Reset call state
                                 println!("  Call {} state transitioned back to IDLE due to failure.", call.index);
                                 // Size adjustment should happen in release_call if used, or outside
                             }
                             _ => {
                                println!("  Ignoring unhandled status code {} from B leg.", code);
                            }
                         }
                     } else { // Status code from A leg? Unexpected in these states.
                        println!("  Ignoring STATUS {} from A leg in state {:?}", method_or_code, call.call_state);
                     }
                 }
                 _ => { // Should not happen
                    println!("  Invalid message_type {} in state {:?}", message_type, call.call_state);
                 }
             }
        }

        CallState::Answered => {
            println!("  Current State: ANSWERED");
            if message_type == REQUEST_METHOD && method_or_code == "ACK" && leg_type == A_LEG {
                println!("  Processing ACK from A leg");
                // Action 4
                // 1. Forward ACK to B leg
                 let b_cseq_val = extract_cseq_number(&call.b_leg_header.cseq).unwrap_or(0);
                 // Corrected format! usage
                 let b_ack = format!(
                    "ACK {} SIP/2.0\r\n\
                    Via: SIP/2.0/UDP {}:{};branch=z9hG4bKackB{}\r\n\
                    {}\r\n\
                    {}\r\n\
                    Call-ID: {}\r\n\
                    CSeq: {} ACK\r\n\
                    Max-Forwards: 70\r\n\
                    User-Agent: TinySIP-Rust\r\n\
                    Content-Length: 0\r\n\r\n", // ACK has no body
                    call.b_leg_contact, // Target URI from B's Contact in 200 OK
                    SIP_SERVER_IP_ADDRESS, SIP_PORT, b_cseq_val, // Unique branch
                    call.b_leg_header.from, // B leg From
                    call.b_leg_header.to, // B leg To
                    call.b_leg_uuid, // B leg Call ID
                    b_cseq_val // Match B's INVITE CSeq num
                 );
                send_sip_message(socket, b_ack.as_bytes(), &call.b_leg_addr.unwrap());

                // 2. Set state to Connected
                 call.call_state = CallState::Connected;
                 println!("  Call {} state transitioned to CONNECTED.", call.index);
             } else if message_type == REQUEST_METHOD && method_or_code == "BYE" {
                 // Handle BYE received *before* ACK (less common, but possible)
                 println!("  Received BYE from leg {} in ANSWERED state (before ACK). Processing BYE.", leg_type);
                 handle_bye(call, message, raw_sip_message, leg_type, socket);
             }
             else {
                 println!("  Ignoring message type {} code/method {} from leg {} in ANSWERED state.", message_type, method_or_code, leg_type);
             }
        }

        CallState::Connected => {
             println!("  Current State: CONNECTED");
             if message_type == REQUEST_METHOD && method_or_code == "BYE" {
                 println!("  Processing BYE from leg {}", leg_type);
                 handle_bye(call, message, raw_sip_message, leg_type, socket);

             }
             // Handle re-INVITE, UPDATE, INFO etc. here if needed
             else {
                 println!("  Ignoring message type {} code/method {} from leg {} in CONNECTED state.", message_type, method_or_code, leg_type);
             }
        }

        CallState::Disconnecting => {
             println!("  Current State: DISCONNECTING");
             if message_type == STATUS_CODE && method_or_code == "200" {
                 // Check if it's 200 OK for BYE or CANCEL
                 if let Some(cseq_val) = get_cseq_header(raw_sip_message) {
                     if cseq_val.contains("BYE") || cseq_val.contains("CANCEL") {
                         println!("  Received 200 OK for BYE/CANCEL from leg {}. Call cleanup.", leg_type);
                         // Action 8: Release call resources
                         CallMap::init_call(call); // Reset call state to Idle/inactive
                         println!("  Call {} state transitioned to IDLE.", call.index);
                         // Size adjustment needed outside or in init_call
                    } else {
                         println!("  Received 200 OK for something other than BYE/CANCEL in DISCONNECTING state. Ignoring.");
                    }
                 }
            }
            // Ignore retransmissions of BYE or other messages while disconnecting
            else {
                println!("  Ignoring message type {} code/method {} from leg {} in DISCONNECTING state.", message_type, method_or_code, leg_type);
            }
        }
    }
}

// Helper to handle BYE processing
fn handle_bye(
    call: &mut Call,
    message: &SipMessage,
    raw_sip_message: &str,
    leg_type: i32,
    socket: &Arc<UdpSocket>,
) {
     // Action 5
    // 1. Send 200 OK for BYE to the sender
    let via_header = get_via_header(raw_sip_message).unwrap_or_default();
    let from_header = get_from_header(raw_sip_message).unwrap_or_default();
    let to_header = get_to_header(raw_sip_message).unwrap_or_default();
    let call_id_header = get_call_id(raw_sip_message).unwrap_or_default();
    let cseq_header = get_cseq_header(raw_sip_message).unwrap_or_default();

    // Corrected format! usage
    let ok_200_bye = format!(
        "SIP/2.0 200 OK\r\n\
        {}\r\n\
        {}\r\n\
        {}\r\n\
        Call-ID: {}\r\n\
        {}\r\n\
        User-Agent: TinySIP-Rust\r\n\
        Content-Length: 0\r\n\r\n",
        via_header, // Via from BYE
        from_header, // From from BYE
        to_header, // To from BYE
        call_id_header, // Call-ID from BYE
        cseq_header // CSeq from BYE
    );
    // Send response back to the source of the BYE
    send_sip_message(socket, ok_200_bye.as_bytes(), &message.client_addr);

    // 2. Construct and send BYE to the *other* leg
    let other_leg_addr;
    let bye_other_leg;

    if leg_type == A_LEG {
        // Send BYE to B leg
        other_leg_addr = call.b_leg_addr.expect("B leg address missing for BYE "); // Added space
        let b_cseq_num = next_cseq();
        let b_branch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        // Corrected format! usage
        bye_other_leg = format!(
            "BYE {} SIP/2.0\r\n\
            Via: SIP/2.0/UDP {}:{};branch=z9hG4bKbyeB{}\r\n\
            {}\r\n\
            {}\r\n\
            Call-ID: {}\r\n\
            CSeq: {} BYE\r\n\
            Max-Forwards: 70\r\n\
            User-Agent: TinySIP-Rust\r\n\
            Content-Length: 0\r\n\r\n",
            call.b_leg_contact, // Target B using its Contact URI
            SIP_SERVER_IP_ADDRESS, SIP_PORT, b_branch, // Server's Via
            call.b_leg_header.from, // Stored B-leg From
            call.b_leg_header.to, // Stored B-leg To
            call.b_leg_uuid, // Stored B-leg Call-ID
            b_cseq_num // New CSeq for BYE
        );

    } else { // leg_type == B_LEG
        // Send BYE to A leg
        other_leg_addr = call.a_leg_addr.expect("A leg address missing for BYE "); // Added space
        let a_cseq_num = next_cseq();
         let a_branch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
         // For BYE to A leg, From is B's original To, To is B's original From
        let bye_a_from = call.a_leg_header.to.replace("To:", "From:"); // Swap To->From
        let bye_a_to = call.a_leg_header.from.replace("From:", "To:"); // Swap From->To

        // Corrected format! usage
        bye_other_leg = format!(
            "BYE {} SIP/2.0\r\n\
            Via: SIP/2.0/UDP {}:{};branch=z9hG4bKbyeA{}\r\n\
            {}\r\n\
            {}\r\n\
            Call-ID: {}\r\n\
            CSeq: {} BYE\r\n\
            Max-Forwards: 70\r\n\
            User-Agent: TinySIP-Rust\r\n\
            Content-Length: 0\r\n\r\n",
            call.a_leg_contact, // Target A using its Contact URI
            SIP_SERVER_IP_ADDRESS, SIP_PORT, a_branch, // Server's Via
            bye_a_to, // Swapped From header
            bye_a_from, // Swapped To header
            call.a_leg_uuid, // A leg Call ID
            a_cseq_num // New CSeq for BYE
        );
    }

    send_sip_message(socket, bye_other_leg.as_bytes(), &other_leg_addr);

    // 3. Set state to Disconnecting
    call.call_state = CallState::Disconnecting;
    // The unterminated string error pointed here, but should be resolved now
    println!("  Call {} state transitioned to DISCONNECTING.", call.index);
}