use futures::executor::block_on;
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

/// Aborts on panic.
pub fn spawn_blocking<T: Send + 'static>(
    f: impl Future03<Output = T> + Send + 'static,
) -> JoinHandle<T> {
    tokio::task::spawn_blocking(move || block_on(abort_on_panic(f)))
}

/// Does not abort on panic, panics result in an `Err` in `JoinHandle`.
pub fn spawn_blocking_allow_panic<R: 'static + Send>(
    f: impl 'static + FnOnce() -> R + Send,
) -> JoinHandle<R> {
    tokio::task::spawn_blocking(f)
}
