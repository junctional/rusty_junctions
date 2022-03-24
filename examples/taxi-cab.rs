/// Simple toy example to demonstrate the basic API of the library.
// The only struct that needs to be brought into score is the Junction itself.
use rusty_junctions::Junction;

fn main() {
    // Create a new Junction.
    let j = Junction::new();

    // Create new channels on the Junction j.
    let name = j.send_channel::<String>();
    let value = j.send_channel::<i32>();
    let get = j.recv_channel::<i32>();

    // Declare a new Join Pattern on the Junction using the channels above.
    // j.when(&name).and(&value).then_do(|n, v| {
    //     println!("{} {}", n, v);
    // });

    j.when_recv(&get).then_do(|| 10);

    let val = get.recv();
    println!("{val:?}");

    // Send all the required messages for the Join Pattern above to fire.
    value.send(1729).unwrap();
    name.send(String::from("Taxi")).unwrap();
}
