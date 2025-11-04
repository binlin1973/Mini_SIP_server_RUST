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
    // Returns a tuple: (Option<index>, leg_type)
    // Takes a guard to allow inspection of the map.
    pub fn find_call_by_callid(
        call_map: &CallMap, // Inspect current map snapshot
        call_id: &str,
    ) -> (Option<usize>, i32) {
        if call_id.is_empty() {
            return (None, 0);
        }

        for (index, call) in call_map.calls.iter().enumerate() {
            if call.is_active {
                if call.a_leg_uuid == call_id {
                    return (Some(index), A_LEG);
                }
                if call.b_leg_uuid == call_id {
                    return (Some(index), B_LEG);
                }
            }
        }
        (None, 0) // Not found
    }

    // Allocates a new call from the map if available.
    // Returns a mutable reference to the newly activated call.
    // Takes a mutable guard.
    // Returns the index of the allocated call.
    pub fn allocate_new_call_mut(call_map_guard: &mut MutexGuard<CallMap>) -> Option<usize> {
        // Return index instead of mutable ref
        if call_map_guard.size >= MAX_CALLS {
            return None; // Map is full
        }

        // Find the first inactive call slot
        for i in 0..MAX_CALLS {
            if !call_map_guard.calls[i].is_active {
                // Reset call state completely before activating
                let mut call = Call {
                    index: i,
                    ..Call::default()
                };
                call.is_active = true; // Mark as active
                call_map_guard.calls[i] = call;
                call_map_guard.size += 1;
                return Some(i); // Return the index
            }
        }
        None // Should not happen if size < MAX_CALLS, but added for safety
    }

    // Resets a call struct to its default (inactive) state.
    // Typically called when a call ends or needs cleanup.
    pub fn release_call(&mut self, index: usize) {
        if let Some(slot) = self.calls.get_mut(index) {
            let was_active = slot.is_active;
            *slot = Call {
                index,
                ..Call::default()
            };
            if was_active {
                self.size = self.size.saturating_sub(1);
            }
        }
    }
}
