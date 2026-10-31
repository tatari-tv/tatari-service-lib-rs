use super::*;

#[test]
fn init_captures_the_start_once() {
    init();
    let first = start().system;
    init();
    assert_eq!(start().system, first);
}

#[test]
fn status_reads_the_process_clock() {
    init();
    let got = status("ok", None);
    assert_eq!(got.status, "ok");
    assert!(got.uptime >= 0.0);
    assert_eq!(got.error, None);
}

#[test]
fn status_passes_a_set_error_through() {
    let got = status("unhealthy", Some("db down".to_string()));
    assert_eq!(got.error.as_deref(), Some("db down"));
}

#[test]
fn version_and_deployed_read_the_real_env() {
    // Reads the real env without mutating it, so only the shape is asserted.
    let build = Build {
        branch: "b",
        revision: "r",
        describe: "d",
    };
    assert!(!version(&build).branch.is_empty());
    assert!(!deployed(false).environment.is_empty());
    assert_eq!(deployed(true).deployer, None);
}

#[test]
fn status_uptime_advances() {
    init();
    let first = status("ok", None).uptime;
    std::thread::sleep(std::time::Duration::from_millis(20));
    let second = status("ok", None).uptime;
    assert!(second > first, "uptime went from {first} to {second}");
}
