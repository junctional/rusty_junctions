/// Simple storage cell implementation to demonstrate every available
/// channel as well as join patterns with repeated channels.
use rusty_junctions::Junction;

fn main() {
    /* Start of the Join Pattern setup. */

    // Declare a new Junction to create new channels and construct new
    // Join Patterns based on them.
    let cell = Junction::new();

    // New channel to retrieve the value of the storage cell.
    let get = cell.recv_channel::<i32>();

    // New channel to update the value of the storage cell.
    let put = cell.send_channel::<i32>();

    // New channel to swap the value of the storage cell for a new one and
    // retrieve the value that was just replaced.
    let swap = cell.bidir_channel::<i32, String>();

    // New channel that will actually carry the value so that at no point
    // any given thread will have possession over it so concurrency issues
    // are avoided by design.
    let val = cell.send_channel::<i32>();

    // Set up some clones of the above channels we can move over to the
    // thread in which the function body of the Join Pattern will run.
    //
    // Clones of channels work like clones of the std::sync::mpsc::Sender
    // clones - any message sent from the clone will be received as if
    // sent from the original.
    let get_val = val.clone();
    let put_val = val.clone();
    let swap_val = val.clone();
    let val_val = val.clone();

    // Declare a new Join Pattern to update the storage cell value. If
    // both the put and val channel have sent a message, meaning someone
    // requested a value update and there is a value to be updated, send
    // a new val message through one of val's clones that carries the
    // updated value.
    cell.when(&put).and(&val).then_do(move |new, _old| {
        println!(">> put-val pattern fired with new={}!", new);
        put_val.send(new).unwrap();
    });

    // Declare a new Join Pattern to retrieve the storage cell value. If
    // both the get and val channel have sent a message, meaning someone
    // requested the value and there is a value to be given, return that
    // value and resend it through one of val's clones so that the value
    // is still available in future and not just consumed once.
    cell.when(&val).and_recv(&get).then_do(move |v| {
        println!(">> val-get pattern fired with v={}!", v);

        get_val.send(v.clone()).unwrap();

        v
    });

    // Declare a new Join Pattern to swap the storage cell value with a
    // new one and retrieve the old. Essentially works like a combination
    // of the previous two Join Patterns, with one crucial distinction:
    // with this Join Pattern, the update of the value and the retrieval
    // of the old are atomic, meaning that it is guaranteed that even in
    // a multithreaded environment with many users accessing the storage
    // cell, the value retrieved is exactly the value that has been
    // updated.
    cell.when(&val).and_bidir(&swap).then_do(move |key, value| {
        println!(
            ">> val-swap pattern fired with old={} and new={}!",
            key, value
        );
        swap_val.send(key).unwrap();

        "Something".to_string()
    });

    // Declare a new Join Pattern that mentions the same channel multiple
    // times, so if the val channel has sent two messages they will be
    // combined into a single messages sent by a clone of val. This ensures
    // that eventually, the storage cell will only keep a single value
    // around.
    cell.when(&val).and(&val).then_do(move |a, b| {
        println!(">> val-val pattern fired with a={} and b={}!", a, b);
        val_val.send(a + b).unwrap();
    });

    /* End of the Join Pattern setup. */

    // Initialise the storage cell by sending an initial value that
    // can be picked up in future executions of the above Join Patterns.
    val.send(1729).unwrap();

    // Request a value update if one is available.
    put.send(42).unwrap();

    // Send another value that will eventually get combined with the
    // existing one.
    val.send(1).unwrap();

    // Request another value update.
    put.send(22).unwrap();

    // Request the current value of the storage cell and print it.
    println!("get.recv()={}", get.recv().unwrap());

    // Request a swap of the current value of the storage cell with a new
    // one and print the old value that is retrieved as a result.
    let thing: String = swap.send_recv(16).expect("faield to get value");
    println!("swap.send_recv()={thing}");

    // Request the current value of the storage cell again and print it.
    println!("get.recv()={}", get.recv().unwrap());
}
