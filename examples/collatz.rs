/// Junction computing the Collatz sequence for a given number up to a
/// maximum number of iterations.
///
/// This example demonstrates how finite iterations can be implemented using
/// junctions and join patterns. It also demonstrates how user-defined types
/// can be used with the channels that this library provides and how to return
/// a final result of an entirely asynchronous calculation. Run the example
/// by executing the following command in the repository root:
///
/// $ cargo run --example collatz -- <initiat value> <maximum iterations>
///
/// replacing the arguments after the "--" with the desired integer values.
/// Refer to the following Wikipedia article for the mathematical background:
///
/// https://en.wikipedia.org/wiki/Collatz_conjecture

use std::env;

use rusty_junctions::Junction;

// Define a result type to be sent over channels.
// Note that the type *must* implement the clone trait.
#[derive(Clone)]
struct CollatzResult {
    final_value: u64,
    iterations_left: u64,
}

// Auxiliary function to make the code a little more readable.
fn is_even(value: u64) -> bool {
    value % 2 == 0
}

fn main() {
    // Get command line input for the initial value of the sequence and the
    // maximum number of iterations to perform.
    let args: Vec<String> = env::args().collect();

    let initial_value: u64 = args[1].parse().unwrap();
    let max_iterations: u64 = args[2].parse().unwrap();

    // Set up the junction to perform the calculations.
    let collatz = Junction::new();

    // Asynchronous state channel to hold the current value in the sequence.
    let value = collatz.send_channel::<u64>();
    // Asynchronous state channel to hold the number of iterations left.
    let iter = collatz.send_channel::<u64>();

    // Channel to signal that the calculation is either finished or that
    // the maximum number of iterations has been reached.
    let finished = collatz.send_channel::<CollatzResult>();
    // Channel to receive the final result of the calculation back to the main
    // thread.
    let result = collatz.recv_channel::<CollatzResult>();

    // Perform the iterations to compute the sequence.
    let value_clone = value.clone();
    let iter_clone = iter.clone();
    let finished_clone = finished.clone();
    collatz.when(&iter).and(&value).then_do(move |n, v| {
        // If there are still iterations left or the sequence has not reached
        // 1 yet, compute the next step and print the transition.
        if n > 0 && v != 1 {
            iter_clone.send(n - 1).unwrap();

            let new_v = if is_even(v) { v / 2 } else { 3 * v + 1 };

            println!("{} -> {}", v, new_v);
            value_clone.send(new_v).unwrap();
        } else {
            // The computation is finished, either by reaching 1 or by running
            // out of available iterations. Send the result with a user-defined
            // type to the main channel.
            finished_clone
                .send(CollatzResult {
                    final_value: v,
                    iterations_left: n,
                })
                .unwrap();
        }
    });

    // If the computation has finished and a thread requested the result, send
    // it to that thread.
    collatz.when(&finished).and_recv(&result).then_do(|res| res);

    // Send the initial values taken from the command line. This starts the
    // iteration, which from here on out happens completely asynchronously.
    value.send(initial_value).unwrap();
    iter.send(max_iterations).unwrap();

    // Wait for the final result to arrive.
    let res = result.recv().unwrap();

    println!(
        "Final result: {} after {} iteration(s)!",
        res.final_value,
        max_iterations - res.iterations_left
    );
}
