use sip_server_rust::sip_defs::{CallMap, A_LEG};
use std::sync::{Arc, Mutex};

fn init_call_map() -> Arc<Mutex<CallMap>> {
    Arc::new(Mutex::new(CallMap::new()))
}

#[test]
fn allocate_increments_size_and_marks_active() {
    let map = init_call_map();
    let mut guard = map.lock().unwrap();
    assert_eq!(guard.size, 0);

    let idx = CallMap::allocate_new_call_mut(&mut guard).expect("should allocate slot");
    assert_eq!(idx, 0);
    assert!(guard.calls[idx].is_active);
    assert_eq!(guard.size, 1);
}

#[test]
fn release_decrements_size_and_reuses_slot() {
    let map = init_call_map();
    let mut guard = map.lock().unwrap();

    let idx = CallMap::allocate_new_call_mut(&mut guard).expect("alloc");
    guard.calls[idx].callee = "callee".into();
    guard.release_call(idx);

    assert_eq!(guard.size, 0);
    assert!(!guard.calls[idx].is_active);
    assert!(guard.calls[idx].callee.is_empty());

    let reused = CallMap::allocate_new_call_mut(&mut guard).expect("re-alloc");
    assert_eq!(reused, idx);
    assert_eq!(guard.size, 1);
}

#[test]
fn release_saturates_size_on_double_free() {
    let map = init_call_map();
    let mut guard = map.lock().unwrap();

    let idx = CallMap::allocate_new_call_mut(&mut guard).expect("alloc");
    guard.release_call(idx);
    guard.release_call(idx); // releasing again should not underflow

    assert_eq!(guard.size, 0);
}

#[test]
fn find_call_by_callid_returns_leg_and_index() {
    let map = init_call_map();
    let mut guard = map.lock().unwrap();
    let idx = CallMap::allocate_new_call_mut(&mut guard).unwrap();
    guard.calls[idx].a_leg_uuid = "call-a".into();

    let (found, leg) = CallMap::find_call_by_callid(&guard, "call-a");
    assert_eq!(found, Some(idx));
    assert_eq!(leg, A_LEG);

    let (missing, _) = CallMap::find_call_by_callid(&guard, "other");
    assert!(missing.is_none());
}
