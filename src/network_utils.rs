
use std::net::UdpSocket;
use std::sync::Arc;

// Sends a SIP message using the main server socket.
// Takes Arc<UdpSocket> to use the shared server socket.
pub fn send_sip_message(
    socket: &Arc<UdpSocket>, // Use the shared server socket
    message_buffer: &[u8],
    destination: &std::net::SocketAddr,
) {
    match socket.send_to(message_buffer, destination) {
        Ok(bytes_sent) => {
            // Optionally log the sent message
            // let msg_str = String::from_utf8_lossy(message_buffer);
            // println!(
            //     "\n================ SENT to {} ================\n{}\n================================================",
            //     destination, msg_str
            // );
             println!("Tx SIP message ({} bytes) to {}", bytes_sent, destination);
             if let Ok(msg_str) = String::from_utf8(message_buffer.to_vec()) {
                if msg_str.len() < 300 { // Print short messages
                    println!("   Content: {}", msg_str.lines().next().unwrap_or(""));
                } else {
                     println!("   Content: {} ... (truncated)", msg_str[..100].lines().next().unwrap_or(""));
                }
             }
        }
        Err(e) => {
            eprintln!("Failed to send message to {}: {}", destination, e);
        }
    }
}