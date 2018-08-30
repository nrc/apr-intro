# Introduction to asynchronous programming in Rust

Software often needs to many things at once - a server serving web pages to multiple clients, GUI software rendering a GUI and doing computation, or an operating system taking care of many hardware tasks and user processes. The typical approach is to use threads, however, at very large scale threads are too costly; they use more memory than necessary and there is a significant cost in context switching. Asynchronous programming is an alternative to threads where concurrency is provided by library and user code.

This book aims to be a comprehensive, up-to-date guide on the async story in Rust, appropriate for beginners and old hands alike. We assume you already know Rust fairly well, including having done some multi-threaded programming. If any Rust terms in this guide are unfamiliar, you should check out [the Rust book](https://doc.rust-lang.org/book/).

In this guide we'll start with high-level programming using `async`/`await`, then gradually descend the technology stack to cover lower-level primitives that can give you more control and flexibility, and libraries for specific tasks: futures, event loops, and asynchronous IO and networking.

If you want to run or experiment with the examples in the introduction, the first few examples are in the [small](examples/small) project, and the echo server is at [echo](examples/echo).


## Models of computation

We'll start by covering a few ways of getting work done on a computer. We'll cover sequential, multi-threaded, and asynchronous execution.

### Sequential

Sequential execution means that tasks are executed one after the other. Single-threaded Rust code is executed sequentially. For example,

```rust
fn main() {
    do_one_thing();
    do_another_thing();
}
```

Here, `do_one_thing` will always be executed before `do_another_thing`, and execution of `do_one_thing` will finish before execution of `do_another_thing` begins.

In a sequential world, tasks cannot interfere with one another, data races don't exist, and execution is deterministic. However, we can only utilize a single processor core, and if we have to wait for IO or another process, we can't get anything else done.


### Multi-threaded

In multi-threaded execution, each task is executed on its own thread. In Rust, threads are operating system threads.

Threads execute concurrently. Depending on the implementation of threads, the number of cores in the processor, and what else is happening on that processor, the tasks may execute in parallel. If the tasks are not executed on different cores, they may be interleaved by the operating system so that both threads can make progress.

E.g.,

```rust
fn main() {
    // Spawn two new threads to do some work.
    let t1 = thread::spawn(|| do_one_thing());
    let t2 = thread::spawn(|| do_another_thing());

    // Wait for both threads to complete.
    t1.join();
    t2.join();
}
```

Here we don't know which task will start or finish first. On the bright side, we can make use of multiple processor cores and one thread can continue even if the other is blocked.


### Asynchronous

Asynchronous programming is concurrent but not parallel. It happens on a single thread (though later we'll see how to use multiple threads too), but if one tasks needs to waiting for something, another task can execute and make progress.

The highest level of abstraction for asynchronous programming uses `async` and `await` syntax. We'll see lower level versions later. `async` marks a function (or block) which can be executed asynchronously. `await` can only be used inside an `async` function (or block), it starts executing asynchronous code and then blocks until that code completes. While a task is blocked in an `await`, other tasks can make progress.

In the following example `do_one_thing_async` and `do_another_thing_async` are `async` functions:

```rust
async fn async_seq() {
    await!(do_one_thing_async());
    await!(do_another_thing_async());
}

fn main() {
    block_on(async_seq()); // Wait for async_seq to complete.
}
```

In `async_seq`, do_another_thing_async call two `async` functions, `await`ing both. The `await`s cause the functions to be called sequentially. Just like in our sequential example, `do_one_thing_async` finishes before `do_another_thing_async` starts.

```rust
async fn async_concurrent() {
    let f1 = do_one_thing_async(1);
    let f2 = do_another_thing_async(2);
    join!(f1, f2);
}

fn main() {
    block_on(async_concurrent()); // Wait for async_concurrent to complete.
}

```

This second example executes the two functions concurrently - their execution will be interleaved. In contrast to the threaded example, the interleaving happens on the same OS thread using just Rust library code, rather than relying on the operating system.

`join!` is like an n-way `await`, it runs both tasks concurrently until both complete.

TODO creating f1 does not execute it

I don't want to get into the implementation of those async functions just yet. But they each have an `await` in their bodies. What happens at runtime is that when it is called, `do_one_thing_async` is executed until we hit it's `await`, then `do_another_thing_async` is executed until we hit *it's* `await`. Then each function is polled in turn to see if it can continue. Once one can it is executed until it completes or hits another `await`, and the process continues until both functions complete.

In practice, `await` is usually used to wait for some IO to complete, e.g., to wait for a packet to be received over a network. So while one task is waiting for input to arrive, other tasks can get some work done. In a high performance server, thousands of tasks can be waiting for and processing input concurrently while only using one OS thread (or one OS thread per core).


### A 'practical' example

To give a larger, more practical example, we're going to implement an echo server using Tokio. Tokio is a fundamental part of Rust's HTTP stack and we'll revisit it in depth later.

An echo server is a program which listens for incoming packets and then sends a copy straight back to the sender.

(Example adapted from [Experimental async / await support for Tokio](https://tokio.rs/blog/2018-08-async-await/)).

Let's look at the implementation one function at a time, starting with `handle` which handles an incoming TCP stream (which corresponds to a TCP connection with a client).

```rust
async fn handle(mut stream: TcpStream) {
    let mut buf = [0; 1024];

    loop {
        match await!(stream.read_async(&mut buf)).unwrap() {
            0 => break, // Socket closed.
            n => {
                // Send the data back.
                await!(stream.write_all_async(&buf[0..n])).unwrap();
            }
        }
    }
}
```

The function is `async`, so that it can be executed asynchronously. After allocating a buffer, it loops while waiting for data to come in on the stream. We use `await` so that while we're waiting other tasks can make progress. Once we've read some data, we write it straight back to the stream, again we do this asynchronously by using `await` - while we are busy writing to the stream, other tasks can make progress.

Next lets look at the code for listening for new connections:

```rust
async fn listen(addr: SocketAddr) {
    let listener = TcpListener::bind(&addr).unwrap();
    let mut incoming = listener.incoming();

    while let Some(stream) = await!(incoming.next()) {
        let stream = stream.unwrap();
        tokio::spawn_async(handle(stream));
    }
}
```

Again, it's an `async` function. We loop while waiting for new connections, again we use `await` so that other tasks can make progress. When we get a new connection we spawn a new task (this is like spawning a thread, but only spawns a new asynchronous task). The new task calls the `handle` function to handle incoming data.

Since `listen` spawns a new task for each connection, we could have multiple tasks running 'at once'. Because we're using asynchronous programming, each task can make progress without blocking the others when waiting for IO, and without the context switching and memory overhead of an OS thread.

For completeness, `main` looks like this:

```rust
fn main() {
    tokio::run_async(listen("127.0.0.1:8080".parse().unwrap()));
}
```

It just passes an address to listen on to `listen` and runs `listen` asynchronously.

To try this out, use `cargo run` to run the example. The server will startup and be listening. From another terminal run `netcat 127.0.0.1 8080`, this will create a new TCP connection. Anything you type should be sent to the echo server and echoed back to the netcat terminal.


## Why use asynchronous techniques

There are two things that make asynchronous programming attractive.

First, it allows you to do more with less. You can use a single OS-level thread to field any number of simultaneous interactions; a single-threaded asynchronous server can scale to handle millions of connections.

Now, some operating systems make it possible to use a large number of OS threads relatively cheaply. But there is still overhead. And this leads to the second benefit of async programming: **by making "tasks" essentially free, it enables highly expressive programming patterns that would be impractical in a synchronous setting.**

In other words, the efficiency gains are so stark that they unlock a powerful new style of programming.

So what's the catch?

Threads are treated in a first class way at the OS level, but if you want to juggle different activities within the same thread, it's all on you. Fortunately, Rust is expressive enough that we can build shared, zero-cost abstractions that make "task-level" programming a first-class concept as well.

That said, there remain some important differences between sync and async programming in Rust. The purpose of this book is, in part, to help guide you through these differences, teaching you a set of patterns for effective async programming.

Finally, it's worth remembering that traditional multi-threaded programming remains quite effective, outside of the highest-scale servers. In particular, Rust's advantages around memory footprint and predictability mean that you can get much farther with synchronous services than in many other languages. As with any other architectural choice, it's important to consider whether your application would be better served by using the simpler synchronous model.
