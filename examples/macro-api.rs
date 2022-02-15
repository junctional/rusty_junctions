use rusty_junctions_client_macro::{channel_def, junction_dec, when, junction};

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
    // Needs to have the Controller explicitly stopped
    handle.stop();


    // Single Junction Procedural Macro API
    junction! {
        name as Send::String,
        value as Recv::String,
        | name, value | {
            println!("Single Junction Procedural Macro API: {name}");
            name
        },
    };
    // value.send(2).unwrap();
    name.send(String::from("Hello, World!")).unwrap();
    let value = value.recv();
    println!("Got value {value:?}");


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
