use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::io;

// Import definitions and modules
mod sip_defs;
mod call_map;
mod worker;
mod network_utils;
mod parsing; // If created separately

use sip_defs::*;
use sip_defs::CallMap;
use worker::process_sip_messages;

fn main() -> io::Result<()> {
    println!("Starting SIP server on port {}...", SIP_PORT);

    // 1. Setup Server Socket (UDP)
    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), SIP_PORT);
    let socket = UdpSocket::bind(bind_addr)?;
    socket.set_nonblocking(true)?; // Set socket to non-blocking
    println!("SIP server socket bound to {}", bind_addr);

    // Wrap socket in Arc for sharing with sender utility (and potentially workers if they sent directly)
    let shared_socket = Arc::new(socket);

    // 2. Initialize Shared Call Map
    let call_map = Arc::new(Mutex::new(CallMap::new()));
    println!("Call map initialized with capacity {}.", MAX_CALLS);

    // 3. Initialize Worker Threads and Queues (Channels)
    let mut worker_handles = Vec::with_capacity(MAX_THREADS);
    let mut worker_senders = Vec::with_capacity(MAX_THREADS);

    for i in 0..MAX_THREADS {
        let (sender, receiver) = mpsc::sync_channel::<SipMessage>(QUEUE_CAPACITY); // Bounded channel
        worker_senders.push(sender);

        let call_map_clone = Arc::clone(&call_map);
        let socket_clone = Arc::clone(&shared_socket); // Clone Arc for worker

        let handle = thread::spawn(move || {
            process_sip_messages(receiver, call_map_clone, socket_clone);
        });
        println!("Worker thread {} created.", i);
        worker_handles.push(handle);
    }

    // 4. Main Server Loop (Receiving Messages)
    let mut buffer = vec![0u8; BUFFER_SIZE + 1]; // Reusable buffer
    let mut next_worker_index = 0;

    println!("Entering main server loop...");
    loop {
        match shared_socket.recv_from(&mut buffer) {
            Ok((bytes_received, client_addr)) => {
                if bytes_received > 0 && bytes_received <= BUFFER_SIZE {
                    // Create SipMessage with a copy of the received data
                    let received_data = buffer[..bytes_received].to_vec();
                    let message = SipMessage {
                        buffer: received_data,
                        client_addr,
                    };

                    // Distribute message to a worker thread (simple round-robin)
                    // Using try_send for bounded channel to avoid blocking main thread
                    match worker_senders[next_worker_index].try_send(message) {
                        Ok(_) => {
                             //println!("Message from {} enqueued to worker {}.", client_addr, next_worker_index);
                        }
                        Err(mpsc::TrySendError::Full(msg)) => {
                            eprintln!(
                                "Worker {} queue full. Dropping message from {}.",
                                next_worker_index, msg.client_addr
                            );
                             // TODO: Maybe send 503 Service Unavailable back?
                        }
                         Err(mpsc::TrySendError::Disconnected(_)) => {
                             eprintln!("Worker {} channel disconnected. Stopping?", next_worker_index);
                             // Handle error, maybe respawn worker or stop server
                             break; // Example: Stop server if worker dies
                         }
                    }

                    next_worker_index = (next_worker_index + 1) % MAX_THREADS;
                } else if bytes_received > BUFFER_SIZE {
                     eprintln!("Received oversized message from {}. Ignored.", client_addr);
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No data available right now, yield CPU briefly
                thread::sleep(std::time::Duration::from_millis(10));
                // Could also use mio or tokio for more efficient polling
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                // Consider stopping or handling specific errors
                // break; // Example: Stop on other errors
                 thread::sleep(std::time::Duration::from_millis(100)); // Avoid busy-looping on persistent errors
            }
        }
        // Add a condition to break the loop for graceful shutdown if needed
    }

    // 5. Cleanup (Optional - current loop is infinite)
    println!("Shutting down server...");
    // Signal workers to stop (e.g., by closing channels or sending a poison pill)
    // Join worker threads
    // for handle in worker_handles {
    //     handle.join().expect("Failed to join worker thread");
    // }
    // println!("All worker threads joined.");

     Ok(()) // Currently unreachable
}