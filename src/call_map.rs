use crate::sip_defs::*;
use std::sync::MutexGuard; // To type hint the lock guard

impl CallMap {
    // Creates a new, empty CallMap initialized with inactive calls
    pub fn new() -> Self {
        let mut calls = Vec::with_capacity(MAX_CALLS);
        for i in 0..MAX_CALLS {
            calls.push(Call {
                index: i,
                is_active: false,
                ..Default::default() // Initialize other fields to default
            });
        }
        CallMap { calls, size: 0 }
    }

    // Finds an *active* call by Call-ID.
    // Returns a tuple: (Option<&mut Call>, leg_type)
    // Takes a mutable guard to allow modification of the found call.
    pub fn find_call_by_callid_mut<'a>(
        call_map_guard: &'a mut MutexGuard<CallMap>, // Pass the guard explicitly
        call_id: &str,
    ) -> (Option<&'a mut Call>, i32) { // Return mutable reference
        if call_id.is_empty() {
            return (None, 0);
        }

        for call in call_map_guard.calls.iter_mut() {
            if call.is_active {
                if call.a_leg_uuid == call_id {
                    return (Some(call), A_LEG);
                }
                if call.b_leg_uuid == call_id {
                    return (Some(call), B_LEG);
                }
            }
        }
        (None, 0) // Not found
    }


    // Allocates a new call from the map if available.
    // Returns a mutable reference to the newly activated call.
    // Takes a mutable guard.
    // Returns the index of the allocated call.
    pub fn allocate_new_call_mut<'a>(
         call_map_guard: &'a mut MutexGuard<CallMap>
    ) -> Option<usize> { // Return index instead of mutable ref
        if call_map_guard.size >= MAX_CALLS {
            return None; // Map is full
        }

        // Find the first inactive call slot
        for i in 0..MAX_CALLS {
            if !call_map_guard.calls[i].is_active {
                // Reset call state completely before activating
                call_map_guard.calls[i] = Call { // Initialize directly
                    index: i,
                    is_active: true, // Mark as active
                     ..Default::default() // Reset all other fields
                };
                call_map_guard.size += 1;
                return Some(i); // Return the index
            }
        }
        None // Should not happen if size < MAX_CALLS, but added for safety
    }

    // Resets a call struct to its default (inactive) state.
    // Typically called when a call ends or needs cleanup.
    // Takes a mutable guard.
    pub fn release_call(call_map_guard: &mut MutexGuard<CallMap>, index: usize) {
        if index < MAX_CALLS && call_map_guard.calls[index].is_active {
             // Reset the call at the given index
            call_map_guard.calls[index] = Call {
                index,
                is_active: false,
                ..Default::default()
            };
            call_map_guard.size -= 1;
        }
    }

     // Helper within CallMap context (no guard needed as it's called with one)
    pub fn init_call(call: &mut Call) {
        *call = Call {
            index: call.index, // Keep the index
            is_active: false, // Mark inactive explicitly during init
            ..Default::default()
        };
    }
}