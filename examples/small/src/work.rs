// Futures are a layer below async/await. We'll need to use some of their support.
use futures::future::poll_fn;

// Some threading primitives.
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::Poll;
use std::thread::{self, sleep};
use std::time::Duration;

// The work here is just waiting for a timeout. We'll print a message before and
// after.
//
// In order to wait for a timeout, we have to start another thread and wait for
// a timer there. Since `sleep` is a synchronous function, the thread is blocked
// until the timeout elapses. If we did this on the main thread, we would block
// all tasks from making progress.
//
// Although we're waiting on another thread to timeout, we're not using threads
// for scheduling the work. If we wanted we could block on async IO instead or
// use a single thread for handling all timeouts. (Writing timers and timeouts
// is surprisingly complicated - https://tokio.rs/blog/2018-03-timers/).
pub async fn do_work_async(x: i32) {
    // Starting up, notify the user.
    println!("starting work {} on thread {:?}", x, thread::current().id());

    // We'll wait for this flag to be set by the timeout thread.
    let flag = Arc::new(AtomicBool::new(false));
    let timeout_flag = flag.clone();

    // Spawn the timeout thread.
    thread::spawn(move || {
        // This thread sleeps, then sets the flag.
        sleep(Duration::from_millis(500));
        timeout_flag.store(true, Ordering::SeqCst);
    });

    // This task will be repeatedly polled until it has completed. We handle that
    // in the below statement. This will all be explained in detail later.
    await!(poll_fn(|lw| {
        if flag.load(Ordering::SeqCst) {
            // Work' is done, notify the user and let the scheduler know we're done.
            println!("work done! {} on thread {:?}", x, thread::current().id());
            Poll::Ready(())
        } else {
            // The timeout has not expired yet. Ask the scheduler to try again
            // later.
            lw.wake();
            Poll::Pending
        }
    }))
}

// A pure sequential version - start, wait, finish.
pub fn do_work(x: i32) {
    println!("starting work {} on thread {:?}", x, thread::current().id());
    sleep(Duration::from_millis(500));
    println!("work done! {} on thread {:?}", x, thread::current().id());
}
