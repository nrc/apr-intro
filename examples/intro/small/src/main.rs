#![feature(async_await, await_macro, futures_api, generators, pin)]

use futures::executor::block_on;
use futures::join;

use std::thread;

// Functions that will do some long-running work.
mod work;

// For each model of computation, we'll run four tasks rather than two from the
// text so there is more opportunity to see reorderings. You'll still probably
// need to run the examples several times.

fn sequential() {
    work::do_work(1);
    work::do_work(2);
    work::do_work(3);
    work::do_work(4);
}

fn multi_threaded() {
    let t1 = thread::spawn(|| work::do_work(1));
    let t2 = thread::spawn(|| work::do_work(2));
    let t3 = thread::spawn(|| work::do_work(3));
    let t4 = thread::spawn(|| work::do_work(4));

    t1.join().unwrap();
    t2.join().unwrap();
    t3.join().unwrap();
    t4.join().unwrap();
}

async fn async_seq() {
    await!(work::do_work_async(1));
    await!(work::do_work_async(2));
    await!(work::do_work_async(3));
    await!(work::do_work_async(4));
}

async fn async_concurrent() {
    let f1 = work::do_work_async(1);
    let f2 = work::do_work_async(2);
    let f3 = work::do_work_async(3);
    let f4 = work::do_work_async(4);
    join!(f1, f2, f3, f4);
}

// It's easiest to see what is happening if you comment out all but one function
// call.
fn main() {
    sequential();

    multi_threaded();

    // The asynchronous versions require us to block on the result to ensure we
    // wait for it to be executed. We can't use `await` here since `main` is not
    // an async function.
    block_on(async_seq());
    block_on(async_concurrent());
}
