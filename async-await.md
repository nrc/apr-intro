# `async` and `await`

In this chapter we'll delve into the `async` and `await` constructs in detail.
`async` is an annotation on functions and blocks which tells the compiler that
the enclosed code should be executed asynchronously. `await` is a macro which
makes the current thread *block* (i.e., wait) until the enclosed asynchronous
code finishes executing.


## Asynchronous computation

Before we jump into the syntax and semantics of asynchronous programming in
Rust, let's try and get an abstract idea of what is going on. Let's imagine
we've got to do some computation and then write it to disk, furthermore lets
assume we've got to do it lots of times and that our input data is independent.
Doing the task sequentially would look something like:

```
for i in 0..lots {
    let result = compute(i);
    write_result_to_disk(result);
}

```

Hopefully it's obvious what is going on here. However, implicit in the above
code is all the waiting that happens. When we read or write to or from disk (or
from a network), there is a long wait while the operating system performs the
IO. Like, a really long wait. In human terms, if a single CPU cycle were to take
one second, then `compute` might take several minutes or hours, but waiting for
IO would take several *days* for an SSD, several *months* for a spinning disk,
and several *years* for an internet round-trip.

If our program is just sitting around waiting during that time, then that is
time wasted. Instead of waiting, we could do more computation.

The program above executes everything sequentially:

```
let result0 = compute(0);
write_result_to_disk(result0);    // start writing 0
                                  // wait ...
                                  //
                                  //
                                  // ... 0 complete
let result1 = compute(1);
write_result_to_disk(result1);    // start writing 1
                                  // wait ...
                                  //
                                  //
                                  // ... 1 complete
let result2 = compute(2);
write_result_to_disk(result2);    // start writing 2
                                  // wait ...
                                  //
                                  //
                                  // ... 2 complete
```

Instead, we could interleave the computations and writes in the waiting time

```
let result0 = compute(0);
write_result_to_disk(result0);    // start writing 0; yield
let result1 = compute(1);         //
write_result_to_disk(result1);    // start writing 1; yield
let result2 = compute(2);         //
                                  // ... 0 complete
write_result_to_disk(result2);    // start writing 2
                                  //
                                  // ... 1 complete
                                  //
                                  //
                                  // ... 2 complete
```

Notice how we finish much quicker (12 lines, c.f., 18, and this effect is
stronger with larger numbers of iteration, tending towards 3n rather than 5n),
and throughput has improved - we started our third task on line 5 rather than
line 13. And all of the above is done on one thread, without any of the costs of
starting, stopping, or switching between threads.

OK, so what does this have to do with `async` and `await`? Well, the compiler
isn't smart enough to turn the above loop into the asynchronous expansion, so we
need to give it some help. We use the `async` keyword to tell the compiler that
a function or block can be executed asynchronously. In the above example we
would declare `write_result_to_disk` as `async fn write_result_to_disk(...)`. We
use the `await` macro to set off an asynchronous task, wait for it to complete,
and (if possible) let the compiler schedule something else while we're waiting.
We might use inside the body of `write_result_to_disk`, e.g.,

```
async fn write_result_to_disk(result: Data) {
    match await!(low_level_write_to_disk(result.as_bytes())) {
        Ok(m) => log!("write {} bytes", n),
        Err(e) => log!("An error occurred: {}", e),
    }
}
```

In the next few sections we'll look at these features in more detail.

## Futures

But before we do that, we'll introduce the abstract idea of futures.

TODO

## The `await` macro

`await` 

## `async`

async functions
    not called immediately
async closures
async blocks


## Programming with async/await

composing the async and await
deadlock
patterns
gotchas

## Reference

RFC: https://github.com/rust-lang/rfcs/pull/2394/files
tracking issue: https://github.com/rust-lang/rust/issues/50547

cite time scales: https://www.prowesscorp.com/computer-latency-at-a-human-scale/


## Implementation

It is often helpful to know how high-level constructs like `async` and `await`
are implemented. We'll go into this in much more detail in later chapters once
we've properly introduced futures.


return type
lifetimes
