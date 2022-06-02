use futures::future::{FutureExt, TryFutureExt};
use std::future::Future as Future03;
use std::panic::AssertUnwindSafe;
use tokio::task::JoinHandle;

fn abort_on_panic<T: Send + 'static>(
    f: impl Future03<Output = T> + Send + 'static,
) -> impl Future03<Output = T> {
    // We're crashing, unwind safety doesn't matter.
    AssertUnwindSafe(f).catch_unwind().unwrap_or_else(|_| {
        println!("Panic in tokio task, aborting!");
        std::process::abort()
    })
}

/// Aborts on panic.
pub fn spawn<T: Send + 'static>(f: impl Future03<Output = T> + Send + 'static) -> JoinHandle<T> {
    tokio::spawn(abort_on_panic(f))
}

pub fn spawn_allow_panic<T: Send + 'static>(
    f: impl Future03<Output = T> + Send + 'static,
) -> JoinHandle<T> {
    tokio::spawn(f)
}

/// Spawns a thread with access to the tokio runtime. Panics if the thread cannot be spawned.
pub fn spawn_thread(
    name: impl Into<String>,
    f: impl 'static + FnOnce() + Send,
) -> std::thread::JoinHandle<()> {
    let conf = std::thread::Builder::new().name(name.into());
    let runtime = tokio::runtime::Handle::current();
    conf.spawn(move || {
        let _runtime_guard = runtime.enter();
        f()
    })
    .unwrap()
}
