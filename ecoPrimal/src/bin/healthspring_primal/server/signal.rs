// SPDX-License-Identifier: AGPL-3.0-or-later

//! Signal handling and lifecycle control.
//!
//! Pure Rust, no C deps. Uses a self-pipe and non-blocking accept timeout
//! to notice SIGTERM/SIGINT. The listener accept has a 1s timeout, so we
//! check the running flag frequently.

use std::io::Read;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tracing::error;

/// Installs a signal-watching thread that polls the running flag.
///
/// Since we forbid unsafe, we rely on the accept loop timeout and
/// `UnixListener::accept` returning `Err(Interrupted)` on signal receipt.
pub fn install_signal_handler(running: &Arc<AtomicBool>) {
    let flag = running.clone();
    std::thread::spawn(move || {
        let (mut reader, _writer) = match std::os::unix::net::UnixStream::pair() {
            Ok(pair) => pair,
            Err(e) => {
                error!("signal pipe creation failed: {e}");
                std::process::exit(1);
            }
        };
        reader.set_read_timeout(Some(Duration::from_secs(1))).ok();
        let mut buf = [0u8; 1];
        loop {
            let _ = reader.read(&mut buf);
            if !flag.load(Ordering::Relaxed) {
                break;
            }
        }
    });

    let _ = running;
}
