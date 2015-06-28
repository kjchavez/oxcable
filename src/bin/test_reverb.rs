//! A simple example using several components
//! Also a lazy, cheeky way to test some simple processors

extern crate oxcable;

#[cfg(not(test))]
fn main() {
    use oxcable::chain::DeviceChain;
    use oxcable::io::audio::AudioEngine;
    use oxcable::reverb::{MoorerReverb, rooms};
    use oxcable::tick::tick_until_enter;

    println!("Initializing signal chain...");
    let engine = AudioEngine::with_buffer_size(256).unwrap();
    let mut chain = DeviceChain::from(
        engine.default_input(1)
    ).into(
        MoorerReverb::new(rooms::HALL, 1.0, -3.0, 0.5, 1)
    ).into(
        engine.default_output(1)
    );

    println!("Playing... Press Enter to quit.");
    tick_until_enter(&mut chain);
    println!("Done!");
}
