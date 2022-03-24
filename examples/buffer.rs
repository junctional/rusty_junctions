/// Example implementation of a rendezvous buffer.
///
/// The values are passed directly from a sender to a receiver. This example
/// most importantly demonstrates error handling related to channels not being
/// able to send messages after their controller has shut down. Failing to
/// correctly handle cases like that will cause threads to panic.
use rusty_junctions::{
    channels::{RecvChannel, SendChannel},
    ControllerHandle, Junction,
};
use std::{any::Any, thread};

// Create a new private buffer Junction and return all required channels.
//
// Note that we're also returning a ControllerHandle so that we can gracefully
// shut down the resources of the private Junction that are running in the
// background.
fn new_buffer<T>() -> (ControllerHandle, SendChannel<T>, RecvChannel<T>)
where
    T: Any + Send,
{
    // Create a new private Junction for the channels and Join Pattern.
    let mut buffer = Junction::new();

    // Asynchronous channel to put a value into the buffer.
    let put = buffer.send_channel::<T>();

    // Synchronous channel to get the value from the buffer.
    let get = buffer.recv_channel::<T>();

    // When there is a value put into the buffer and someone signalled they are
    // ready to get, return the value.
    buffer.when(&put).and_recv(&get).then_do(|v| v);

    // Take ownership of the ControllerHandle so that the Junction does not
    // drop its resources when going out of scope.
    //
    // We want to necessary resources for the buffer to still run in the
    // background, but don't need the Junction itself anymore, since we have
    // created all the necessary channels and Join Patterns.
    let ch = buffer.controller_handle().unwrap();

    // Return the ControllerHandle and the channels to interact with the
    // buffer.
    (ch, put, get)
}

fn main() {
    // Get a new u32 buffer.
    let (mut ch, put, get) = new_buffer::<u32>();

    // Create a new thread interacting with the buffer.
    //
    // Note that we clone the channels here so we can move them. Channels
    // can only be moved to other threads, but the clones of channels behave
    // and interact with the Junction just as the originals.
    let put_1 = put.clone();
    let get_1 = get.clone();
    thread::spawn(move || {
        println!("<Thread 1> Asking for the buffer value...");

        // Ensure that a value is received. If not, shut down the thread.
        // A reason for no value being received is that the controller
        // running in the background has already been asked to shut down.
        // In this case, there is nothing else to do as the channels are
        // now invalid.
        if let Ok(v) = get_1.recv() {
            println!("<Thread 1> Got {}!", v);
            println!("<Thread 1> Putting double the value in...");

            let put_value = 2 * v;
            put_1.send(put_value).unwrap();
            println!("<Thread 1> Done putting {}!", put_value);
        } else {
            println!("<Thread 1> Couldn't get a value anymore...");
            println!("<Thread 1> Shutting down...");
        }
    });

    // Create another thread interacting with the buffer.
    let put_2 = put.clone();
    let get_2 = get.clone();
    thread::spawn(move || {
        println!("<Thread 2> Asking for the buffer value...");
        if let Ok(v) = get_2.recv() {
            println!("<Thread 2> Got {}!", v);
            println!("<Thread 2> Putting three times plus one of the value in...");

            let put_value = 3 * v + 1;
            put_2.send(put_value).unwrap();
            println!("<Thread 2> Done putting {}!", put_value);
        } else {
            println!("<Thread 2> Couldn't get a value anymore...");
            println!("<Thread 2> Shutting down...");
        }
    });

    // Putting an initial value into the buffer.
    //
    // This value will definitely be the initial value as the other two threads
    // start off by getting a value from a synchronous channel, i.e. a channel
    // that blocks until a Join Pattern that it belongs to has fired and
    // provided a value for it.
    let initial_value = 127;
    println!("<Main Thread> Putting an initial value...");
    put.send(initial_value).unwrap();
    println!("<Main Thread> Done putting {}!", initial_value);

    // Get the value from the buffer.
    //
    // It now depends on the internal timings which value will be received
    // here. The only guarantee is that a value will eventual be received,
    // as we have as many puts as we have gets.
    println!("<Main Thread> Asking for the buffer value...");
    if let Ok(v) = get.recv() {
        println!("<Main Thread> Got {}!", v);
    } else {
        println!("<Main Thread> Couldn't get a value anymore...");
    }

    // Clean up the resources running in the background that manage the
    // private buffer Junction.
    // Note that this could happen while either of the other threads are still
    // trying to send messages to the controller that is being cleaned up here.
    ch.stop();
}
