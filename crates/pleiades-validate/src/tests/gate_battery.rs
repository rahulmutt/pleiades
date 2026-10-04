//! Ordering and first-error tests for the concurrent release battery runner
//! (`render::cli::run_gates_concurrently`, FU-23 (r)), with injected gates.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::render::cli::run_gates_concurrently;

type Gate<'a> = &'a (dyn Fn() -> Result<(), String> + Sync);

#[test]
fn concurrent_gates_report_the_first_failure_in_order_even_when_it_finishes_last() {
    let later_failed = AtomicBool::new(false);
    let finished = Mutex::new(Vec::new());
    let first = || {
        // Hold the first gate open until the later gate has already failed,
        // so the first failure in battery order is the last to finish.
        let deadline = Instant::now() + Duration::from_secs(30);
        while !later_failed.load(Ordering::Acquire) {
            assert!(Instant::now() < deadline, "later gate never ran");
            std::thread::sleep(Duration::from_millis(1));
        }
        finished.lock().unwrap().push("first");
        Err("first error".to_string())
    };
    let passing = || Ok(());
    let later = || {
        finished.lock().unwrap().push("later");
        later_failed.store(true, Ordering::Release);
        Err("later error".to_string())
    };
    let gates: [(&str, Gate); 3] = [
        ("first gate failed", &first),
        ("passing gate failed", &passing),
        ("later gate failed", &later),
    ];

    let outcome = run_gates_concurrently(&gates);

    assert_eq!(outcome, Err("first gate failed: first error".to_string()));
    assert_eq!(*finished.lock().unwrap(), ["later", "first"]);
}

#[test]
fn concurrent_gates_pass_when_every_gate_passes() {
    let pass = || Ok(());
    let gates: [(&str, Gate); 3] = [
        ("a failed", &pass),
        ("b failed", &pass),
        ("c failed", &pass),
    ];
    assert_eq!(run_gates_concurrently(&gates), Ok(()));
}

#[test]
fn concurrent_gates_pass_with_no_gates() {
    let gates: [(&str, Gate); 0] = [];
    assert_eq!(run_gates_concurrently(&gates), Ok(()));
}
