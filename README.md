# Rusty Junctions

_Rusty Junctions_ is a Rust crate developed during my bachelor thesis. It aims to implement Join Patterns from the Join Calculus developed by Cédric Fournet and Georges Gonthier [1] in Rust version 1.35.0.

On top of Join Patterns, this library introduces the concept of a _Junction_, which takes a similar place as classes in Polyphonic C# [2]. Junctions act as an overarching structure to Join Patterns and the channels that they are declared with. Junctions group channels together that are used for a combined set of Join Patterns and provide the locality property with respect to contention that the Join Calculus exhibits [1].

## Usage

Those familiar with Join Patterns and their programming paradigm should find it relatively simple to use this crate. Join Patterns are created dynamically with Rusty Junctions and the process follows the same three steps every time:

1. Create a new `Junction`.
2. Create one or more new channels on the `Junction`.
3. Declare one or more new Join Patterns on the `Junction`.

Due to the dynamic nature of the Join Pattern declaration, as soon as a pattern is declared it can be fired. In order to trigger a Join Pattern to fire, all that needs to be done is for each channel involved in the declaration of a Join Pattern to send at least one message. No manual coordination efforts required, once the Junction notices that all necessary messages have been received to fire a particular Join Pattern, it will do so.

Below is a very short example showcasing the basic usage of the library:

```Rust
// The only struct that needs to be brought into score is the Junction itself.
use rusty_junctions::Junction;

fn main() {
    // Create a new Junction.
    let j = Junction::new();

    // Create new channels on the Junction j.
    let name = j.send_channel::<String>();
    let value = j.send_channel::<i32>();

    // Declare a new Join Pattern on the Junction using the channels above.
    j.when(&name).and(&value).then_do(|n, v| { println!("{} {}", n, v); });

    // Send all the required messages for the Join Pattern above to fire.
    value.send(1729).unwrap();
    name.send(String::from("Taxi")).unwrap();
}
```

This and more complex examples can also be found in the [`examples`](https://github.com/smueksch/rusty_junctions/tree/master/examples) folder in this repository.

## Special Thanks

I would like to thank my thesis supervisor [Dr. Ian Stark](http://homepages.inf.ed.ac.uk/stark/), who initially proposed the thesis topic that led to this library. Without him, his constant support and invaluable inputs to solve crucial challenges, none of this would have been possible.

## References

[1] https://www.microsoft.com/en-us/research/wp-content/uploads/2017/01/join-tutorial.pdf<br />
[2] https://dl.acm.org/doi/abs/10.1145/1018203.1018205
