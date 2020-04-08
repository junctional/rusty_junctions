use std::thread;

use rusty_junctions::Junction;
use rusty_junctions::types::ControllerHandle;
use rusty_junctions::channels::{RecvChannel, BidirChannel};

// Set up a private Junction for a rendezvous and return the public channels.
pub fn rendezvous() -> (
    ControllerHandle, BidirChannel<u32, ()>, RecvChannel<()>
) {
    let mut j = Junction::new();

    // Asynchronous token channel to carry the state.
    let token = j.send_channel::<u32>();

    // Synchronous entry channel.
    let entry = j.recv_channel::<()>();

    // Synchronous channel to set up number of available tokens.
    let accept_n = j.bidir_channel::<u32, ()>();

    // Synchronous wait channel.
    let wait = j.recv_channel::<()>();

    // Asynchronous all_gone channel.
    let all_gone = j.send_channel::<()>();

    // Count down the arrivals.
    let token_clone = token.clone();
    let all_gone_clone = all_gone.clone();
    j.when(&token).and_recv(&entry).then_do(move |n| {
        if n == 1 {
            all_gone_clone.send(()).unwrap();
        } else {
            token_clone.send(n - 1);
        }
    });

    // Spawn n new token and wait for all entries to rendezvous.
    let token_clone = token.clone();
    let wait_clone = wait.clone();
    j.when_bidir(&accept_n).then_do(move |n| {
        token_clone.send(n).unwrap();
        wait_clone.recv().unwrap();
    });

    // Stop waiting once all tokens are gone.
    j.when(&all_gone).and_recv(&wait).then_do(|_| {});

    // Prevent Junction from stopping control thread after return.
    let controller_handle = j.controller_handle().unwrap();

    // Return the necessary channels.
    (controller_handle, accept_n, entry)
}

fn main() {
    // Number of entries to rendezvous.
    let num_entries = 3;

    println!("Constructing rendezvous...");
    let (mut ch, accept_n, entry) = rendezvous();
    println!("Done constructing rendezvous!");

    for i in 0..num_entries {
        let entry_clone = entry.clone();
        thread::spawn(move || {
            println!("Sending entry...");
            entry_clone.recv().unwrap();
            println!("Entry accepted!");
        });
    }

    println!("Starting to wait for {} entrie(s)...", num_entries);
    accept_n.send_recv(num_entries).unwrap();
    println!("All {} entrie(s) arrived!", num_entries);

    // Clean up controller resources running in background manually.
    ch.stop();
}
