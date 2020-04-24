/// Santa Claus Problem in Rusty Junctions.
///
/// The code below is based on the paper "Jingle Bells:  Solving the Santa Claus
/// Problem in Polyphonic C#" by Nick Benton
///
/// The paper: https://www.researchgate.net/profile/Nick_Benton2/publication/2569067_Jingle_Bells_Solving_the_Santa_Claus_Problem_in/links/0c9605264f92520a08000000/Jingle-Bells-Solving-the-Santa-Claus-Problem-in.pdf

use rand::Rng;

use std::{thread, time::Duration};

use rusty_junctions::channels::{BidirChannel, RecvChannel};
use rusty_junctions::types::ControllerHandle;
use rusty_junctions::Junction;

fn main() {
    /*****************************
     * Elves Junction & Channels *
     *****************************/

    // Elf Junction.
    let elves = Junction::new();

    // Synchronous channel to signal that elf wants to queue up.
    let elf_queue = elves.recv_channel::<()>();

    // Asynchronous channel to carry the number of elves that are queued up.
    let elves_waiting = elves.send_channel::<u32>();

    /********************************
     * Reindeer Junction & Channels *
     ********************************/

    // Reindeer Junction.
    let reindeer = Junction::new();

    // Synchronous channel to signal that reindeer is back from holiday.
    let reindeer_back = reindeer.recv_channel::<()>();

    // Asynchronous channel to carry the number of reindeer waiting in the stable.
    let reindeer_waiting = reindeer.send_channel::<u32>();

    /*****************************
     * Santa Junction & Channels *
     *****************************/

    // Santa's Junction.
    let santa = Junction::new();

    // Synchronous channel to wait to be woken by either reindeer or elves.
    let wait_to_be_woken = santa.recv_channel::<()>();

    // Asynchronous channel to signal that enough reindeer are ready.
    let reindeer_ready = santa.send_channel::<()>();

    // Asynchronous channel to signal that not enough reindeer are ready.
    // Used for prioritisation.
    let reindeer_not_ready = santa.send_channel::<()>();

    // Synchronous channel to match and consume a reindeer_not_ready message.
    // Used for prioritisation.
    let clear_reindeer_not_ready = santa.recv_channel::<()>();

    // Asynchronous channel to signal that enough elves are ready.
    let elves_ready = santa.send_channel::<()>();

    // Rendezvous channels to let elves into room.
    let (mut ch_1, room_in_accept_n, room_in_entry) = rendezvous();

    // Rendezvous channels to let elves out of room.
    let (mut ch_2, room_out_accept_n, room_out_entry) = rendezvous();

    // Rendezvous channels to harness the reindeer.
    let (mut ch_3, harness_accept_n, harness_entry) = rendezvous();

    // Rendezvous channels to unharness the reindeer.
    let (mut ch_4, unharness_accept_n, unharness_entry) = rendezvous();

    /***********************
     * Elves Join Patterns *
     ***********************/

    // Count up how many elves are waiting and possibly send ready message.
    let elves_ready_clone = elves_ready.clone();
    let elves_waiting_clone = elves_waiting.clone();
    elves
        .when(&elves_waiting)
        .and_recv(&elf_queue)
        .then_do(move |e| {
            if e == 2 {
                // Last elf just queued.
                elves_ready_clone.send(()).unwrap();
                println!("<Elves> Group of 3 ready!");
            } else {
                elves_waiting_clone.send(e + 1).unwrap();
                println!("<Elves> {} waiting!", e + 1);
            }
        });

    /**************************
     * Reindeer Join Patterns *
     **************************/

    // Count up how many reindeer are waiting and possibly send ready message.
    let reindeer_ready_clone = reindeer_ready.clone();
    let reindeer_waiting_clone = reindeer_waiting.clone();
    let clear_reindeer_not_ready_clone = clear_reindeer_not_ready.clone();
    reindeer
        .when(&reindeer_waiting)
        .and_recv(&reindeer_back)
        .then_do(move |r| {
            if r == 8 {
                // Last reindeer just came back.
                clear_reindeer_not_ready_clone.recv().unwrap();
                reindeer_ready_clone.send(()).unwrap();
                println!("<Reindeer> All 9 assembled!");
            } else {
                reindeer_waiting_clone.send(r + 1).unwrap();
                println!("<Reindeer> {} waiting!", r + 1);
            }
        });

    /***********************
     * Santa Join Patterns *
     ***********************/

    // Enough elves are ready so let's consult with them.
    let reindeer_not_ready_clone = reindeer_not_ready.clone();
    let elves_waiting_clone = elves_waiting.clone();
    santa
        .when(&elves_ready)
        .and(&reindeer_not_ready)
        .and_recv(&wait_to_be_woken)
        .then_do(move |_, _| {
            let mut rng = rand::thread_rng();

            // Reindeer will still not be ready so resend just consumed message.
            reindeer_not_ready_clone.send(()).unwrap();

            // Show 3 elves into the office once all are ready.
            println!("<Santa> Woken by elves, now showing them in!");
            room_in_accept_n.send_recv(3).unwrap();
            println!("<Santa> Elf group shown in!");

            // Reset how many elves are waiting to allow others to form a group.
            elves_waiting_clone.send(0).unwrap();

            // Consult with elves for 0 to 10 seconds, i.e. pause thread.
            println!("<Santa> Now consulting with elves!");
            thread::sleep(Duration::from_secs(rng.gen_range(0, 10)));
            println!("<Santa> Consulted with elves!");

            // Done consulting with elves so show all 3 out once all are ready.
            println!("<Santa> Now showing out elves!");
            room_out_accept_n.send_recv(3).unwrap();
            println!("<Santa> Elf group shown out!");
        });

    // Enough reindeer are ready so let's deliver some presents.
    let reindeer_not_ready_clone = reindeer_not_ready.clone();
    let reindeer_waiting_clone = reindeer_waiting.clone();
    santa
        .when(&reindeer_ready)
        .and_recv(&wait_to_be_woken)
        .then_do(move |_| {
            let mut rng = rand::thread_rng();

            // Harness all 9 reindeer once they are all ready.
            println!("<Santa> Woken by reindeer, now harnessing them!");
            harness_accept_n.send_recv(9).unwrap();
            println!("<Santa> Reindeer harnessed!");

            // Reindeer are harnessed, so they are no longer ready.
            // Used for prioritisation.
            reindeer_not_ready_clone.send(()).unwrap();

            // Reset how many reindeer are waiting.
            reindeer_waiting_clone.send(0).unwrap();

            // Deliver toys with reindeer for 0 to 10 seconds, i.e. pause thread.
            println!("<Santa> Now delivering toys!");
            thread::sleep(Duration::from_secs(rng.gen_range(0, 10)));
            println!("<Santa> Toys delivered!");

            // Done delivering toys, unharness all 9 reindeer once all are ready.
            println!("<Santa> Now unharnessing reindeer!");
            unharness_accept_n.send_recv(9).unwrap();
            println!("<Santa> Reindeer unharnessed!");
        });

    // Clear the reindeer_not_ready message. Used for prioritisation.
    santa
        .when(&reindeer_not_ready)
        .and_recv(&clear_reindeer_not_ready)
        .then_do(|_| {});

    /*******************************
     * Start North Pole Operations *
     *******************************/

    // Spawn in the 10 elves and send the initial number of waiting ones.
    for i in 0..10 {
        new_elf(
            elf_queue.clone(),
            room_in_entry.clone(),
            room_out_entry.clone(),
        );
    }
    elves_waiting.send(0).unwrap();

    // Spawn in the 9 reindeer, send the initial number of waiting ones and
    // send that they are not ready yet.
    for i in 0..9 {
        new_reindeer(
            reindeer_back.clone(),
            harness_entry.clone(),
            unharness_entry.clone(),
        );
    }
    reindeer_waiting.send(0).unwrap();
    reindeer_not_ready.send(()).unwrap();

    // Santa keeps napping until something comes up.
    println!("<North Pole> Starting operations!");
    while true {
        println!("<Santa> Starting a nap, waiting to be woken...");
        wait_to_be_woken.recv().unwrap();
        println!("<Santa> Woken from nap!");
    }

    // Clean up the controller resources in the background manually at the end.
    ch_1.stop();
    ch_2.stop();
    ch_3.stop();
    ch_4.stop();
}

// Create a new elf in a new thread.
fn new_elf(
    queue: RecvChannel<()>,
    room_in_entry: RecvChannel<()>,
    room_out_entry: RecvChannel<()>,
) {
    println!("<North Pole> New elf hired!");
    thread::spawn(move || {
        // Random number generator for working and consulting times.
        let mut rng = rand::thread_rng();

        while true {
            // Work for 0 to 10 seconds, i.e. pause thread.
            println!("<Elf> Going to work now!");
            thread::sleep(Duration::from_secs(rng.gen_range(0, 10)));
            println!("<Elf> Done working!");

            // Done working, so wait and try to join a group of 3 elves.
            println!("<Elf> Queuing now, waiting for more elves...");
            queue.recv().unwrap();
            println!("<Elf> Found a group of elves!");

            // Found group of elves so sait for Santa to show me in.
            println!("<Elf> Waiting for Santa to show me in...");
            room_in_entry.recv().unwrap();
            println!("<Elf> Santa is showing me in now!");

            // Consult with Santa for 0 to 10 seconds, i.e. pause thread.
            println!("<Elf> Consulting with Santa now!");
            thread::sleep(Duration::from_secs(rng.gen_range(0, 10)));
            println!("<Elf> Done consulting with Santa!");

            // Wait for Santa to show me out.
            println!("<Elf> Waiting for Santa to show me out...");
            room_out_entry.recv().unwrap();
            println!("<Elf> Santa is showing me out now!");
        }
    });
}

// Create a new reindeer in a new thread.
fn new_reindeer(
    reindeer_back: RecvChannel<()>,
    harness_entry: RecvChannel<()>,
    unharness_entry: RecvChannel<()>,
) {
    println!("<North Pole> New reindeer hired!");
    thread::spawn(move || {
        // Random number generator for holiday and delivery times.
        let mut rng = rand::thread_rng();

        while true {
            // Go on holiday for 0 to 10 seconds, i.e. pause thread.
            println!("<Reindeer> Going on holiday now!");
            thread::sleep(Duration::from_secs(rng.gen_range(0, 10)));
            println!("<Reindeer> Back from holiday!");

            // Done with holiday so join a group of reindeer in the stable.
            println!("<Reindeer> Waiting in stable now for enough reindeer...");
            reindeer_back.recv().unwrap();
            println!("<Reindeer> Reindeer now assembled in stable!");

            // Enough reindeers in the stable so wait for Santa to harness me.
            println!("<Reindeer> Waiting for Santa to harness me...");
            harness_entry.recv().unwrap();
            println!("<Reindeer> Santa is harnessing me now!");

            // Deliver toys with Santa for 0 to 10 seconds, i.e. pause thread.
            println!("<Reindeer> Delivering toys with Santa now!");
            thread::sleep(Duration::from_secs(rng.gen_range(0, 10)));
            println!("<Reindeer> Done delivering toys with Santa!");

            // Done delivering toys with Santa so sait for him to unharness me.
            println!("<Reindeer> Waiting for Santa to unharness me...");
            unharness_entry.recv().unwrap();
            println!("<Reindeer> Santa is unharnessing me now!");
        }
    });
}

// Set up a private Junction for a rendezvous and return the public channels.
pub fn rendezvous() -> (ControllerHandle, BidirChannel<u32, ()>, RecvChannel<()>) {
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
