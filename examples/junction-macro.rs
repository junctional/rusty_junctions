use rusty_junctions_macro::client::junction;

fn main() {
    pretty_env_logger::init();

    junction! {
        message as Send::String,
        id as Send::i32,
        | message, id | {
            println!("Single Junction Procedural Macro API: {message} {id}");
        },
    };
    message
        .send(String::from("Secret Concurrent Message"))
        .unwrap();
    id.send(1960).unwrap();

    junction! {
        main_junction as Junction,
        name as Send::String,
        value as Send::i32,
        | name, value | {
            std::thread::sleep(std::time::Duration::from_secs(5));
            println!("Single Junction Procedural Macro API: {name} {value}");
        },
    };

    value.send(2).unwrap();
    name.send(String::from("Hello, World!")).unwrap();

    let channel = main_junction.send_channel::<String>();
    main_junction.when(&channel).then_do(|c| {
        println!("Newly Installed Pattern: {c}");
    });
    channel.send("Lots of opportunities".to_string()).unwrap();
}
