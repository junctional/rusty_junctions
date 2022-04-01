/// Example implementation of a mutually exclusive lock, or mutex.
///
/// This code provides evidence that concurrent code using locks can equally
/// be implemented using junctions and join patterns.
use rusty_junctions::{
    channels::{RecvChannel, SendChannel},
    ControllerHandle, Junction,
};
use std::thread;

// Create a new mutex using a private junction and return channels to acquire
// and release it.
fn new_mutex() -> (ControllerHandle, RecvChannel<()>, SendChannel<()>) {
    // Private junction to set up the mutex.
    let mut mutex = Junction::new();

    // Channel to acquire the lock. Blocks until it is acquired.
    let acquire = mutex.recv_channel::<()>();
    // Channel to release the lock. Does not block.
    let release = mutex.send_channel::<()>();
    // Asynchronous state channel to represent a lock that can be consumed
    // and released.
    let lock = mutex.send_channel::<()>();

    // When there is a lock available and a thread wants to acquire it,
    // unblock that thread which is equivalent to acquiring the lock.
    // It is mutually exclusive as no new lock message is sent out.
    mutex.when(&lock).and_recv(&acquire).then_do(|_| {});

    // If a thread calls to release the lock, send out a new lock message
    // to unblock any open acquire call.
    let lock_clone = lock.clone();
    mutex.when(&release).then_do(move |_| {
        let _ = lock_clone.send(());
    });

    // Create a lock to be acquired and released.
    lock.send(()).unwrap();

    // Ensure that the controller managing the mutex in the background stays
    // alive after the return below.
    let ch = mutex.controller_handle().unwrap();

    // Return channels to access the lock.
    (ch, acquire, release)
}

fn main() {
    // Create a new lock.
    let (mut ch, acquire, release) = new_mutex();

    // Count up even numbers.
    let acquire_1 = acquire.clone();
    let release_1 = release.clone();
    let jh_1 = thread::spawn(move || {
        for i in 0..11 {
            if i % 2 == 0 && acquire_1.recv().is_ok() {
                println!("<Even> {}", i);
                let _ = release_1.send(());
            }
        }
    });

    // Count up odd numbers.
    let acquire_2 = acquire.clone();
    let release_2 = release.clone();
    let jh_2 = thread::spawn(move || {
        for i in 0..10 {
            if i % 2 != 0 && acquire_2.recv().is_ok() {
                println!("<Odd>  {}", i);
                let _ = release_2.send(());
            }
        }
    });

    // Join the threads and shut down the mutex controller running in the
    // background.
    jh_1.join().unwrap();
    jh_2.join().unwrap();
    ch.stop();
}
