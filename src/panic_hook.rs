use std::{backtrace::Backtrace, panic::PanicHookInfo};

fn trace(info: &PanicHookInfo, backtrace: Backtrace, s: &str) {
    tracing::error!(
        "panicked at {}:{}:{}, reason: \"{}\"",
        info.location().unwrap().file(),
        info.location().unwrap().line(),
        info.location().unwrap().column(),
        s,
    );
    tracing::error!("panic backtrace: {}", backtrace);
}

/// Logs the panic location, reason and backtrace using tracing.
pub fn panic_hook(info: &PanicHookInfo) {
    let backtrace = std::backtrace::Backtrace::force_capture();

    if let Some(s) = info.payload().downcast_ref::<&str>() {
        trace(info, backtrace, s);
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        trace(info, backtrace, s);
    }
}
