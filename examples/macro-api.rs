use rusty_junctions_macro::client::{channel, junction, junction_dec, when};

fn main() {
    pretty_env_logger::init();

    // Standard API
    let junction = rusty_junctions::Junction::new();
    let name = junction.send_channel::<String>();
    let value = junction.send_channel::<i32>();
    junction.when(&name).and(&value).then_do(|name, value| {
        println!("Standard API: {name} {value}");
    });
    value.send(0).unwrap();
    name.send(String::from("Hello, World!")).unwrap();

    // Single Junction Declarative Macro API
    let (name, value, mut handle) = junction_dec! {
        name as Send::String,
        value as Send::i32,
        |name, value| {
            println!("Single Junction Declarative Macro API: {name} {value}");
        },
    };
    value.send(1).unwrap();
    name.send(String::from("Hello, World!")).unwrap();
    // Needs to have the Controller explicitly stopped, if we allowed it to
    // be dropped from the inner scope there would be no guarantee it would
    // have time for the pattern to fire.
    handle.stop();

    // Single Junction Procedural Macro API
    // junction as ControllerHandle, // Bring the cotnroller handle into scope with this name
    junction! {
        // some_junction as Junction,
        get as Recv::i32,
        set as Send::i32,
        value as Send::i32,
        | get, value | {
            println!("Getting value: {value}");
            value_super.send(value).unwrap();
            value
        },
        | set, value | {
            println!("Setting value: {value} --> {set}");
            value_super.send(set).unwrap();
        },
    };
    // let _handle = some_junction.controller_handle();

    value.send(1809124).unwrap();
    let _v = get.recv().unwrap();
    set.send(2022).unwrap();

    // let value = value.recv();
    // println!("Got value {value:?}");

    // When! Macro API
    let junction = rusty_junctions::Junction::new();
    let name = junction.send_channel::<String>();
    let value = junction.send_channel::<i32>();
    when!(junction; name, value).then_do(|name, value| {
        println!("when! Macro API: {name} {value}");
    });
    value.send(3).unwrap();
    name.send(String::from("Hello, World!")).unwrap();
}
