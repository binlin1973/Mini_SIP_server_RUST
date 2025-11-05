use sip_server_rust::sip_defs::{CallMap, MAX_CALLS};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;

fn init_map() -> Arc<Mutex<CallMap>> {
    Arc::new(Mutex::new(CallMap::new()))
}

#[test]
fn allocate_new_call_mut_handles_capacity_and_reuse() {
    let map = init_map();
    let mut guard = map.lock().expect("mutex poisoned");

    assert_eq!(guard.size, 0, "fresh map should start empty");

    let mut indices = Vec::with_capacity(MAX_CALLS);
    for _ in 0..MAX_CALLS {
        let idx = CallMap::allocate_new_call_mut(&mut guard)
            .expect("allocation should succeed while below capacity");
        assert!(idx < MAX_CALLS);
        indices.push(idx);
    }

    assert_eq!(guard.size, MAX_CALLS, "size should match number of active calls");
    assert_eq!(
        indices.len(),
        HashSet::<usize>::from_iter(indices.iter().copied()).len(),
        "indices should be unique while filling up"
    );

    let overflow = CallMap::allocate_new_call_mut(&mut guard);
    assert!(
        overflow.is_none(),
        "allocating beyond capacity should return None"
    );

    let first_idx = indices[0];
    guard.release_call(first_idx);
    assert_eq!(
        guard.size,
        MAX_CALLS - 1,
        "releasing one slot should decrement size"
    );

    let reused_idx = CallMap::allocate_new_call_mut(&mut guard)
        .expect("slot freed moments ago should be reusable");
    assert_eq!(
        reused_idx, first_idx,
        "allocator should reuse the lowest available slot"
    );
    assert_eq!(
        guard.size, MAX_CALLS,
        "size should not exceed capacity after reuse"
    );
}

#[test]
fn release_call_behaves_safely_for_repeated_calls() {
    let map = init_map();
    let mut guard = map.lock().expect("mutex poisoned");

    let idx = CallMap::allocate_new_call_mut(&mut guard).expect("initial allocation");
    let other = CallMap::allocate_new_call_mut(&mut guard).expect("second allocation");
    assert_eq!(guard.size, 2);

    guard.release_call(idx);
    assert_eq!(guard.size, 1, "releasing active slot should decrement size");

    guard.release_call(idx);
    assert_eq!(
        guard.size, 1,
        "releasing inactive slot should not decrement further"
    );

    guard.release_call(other);
    assert_eq!(guard.size, 0, "releasing last active slot should hit zero");
}

#[test]
fn threaded_allocate_release_keeps_size_invariant() {
    let map = init_map();
    let threads: Vec<_> = (0..6)
        .map(|_| {
            let map = Arc::clone(&map);
            thread::spawn(move || {
                for _ in 0..250 {
                    let mut guard = map.lock().expect("mutex poisoned");
                    if let Some(idx) = CallMap::allocate_new_call_mut(&mut guard) {
                        guard.release_call(idx);
                    }
                }
            })
        })
        .collect();

    for handle in threads {
        handle.join().expect("thread should not panic");
    }

    let guard = map.lock().expect("mutex poisoned");
    assert_eq!(guard.size, 0, "all allocations should be released");
}
