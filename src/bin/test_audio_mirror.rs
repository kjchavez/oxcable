//! Reads input from a microphone and mirrors it to a speaker ad nauseum.

extern crate oxcable;

#[cfg(not(test))]
fn main() {
    use oxcable::init;
    use oxcable::types::Device;
    use oxcable::io::audio::{AudioIn, AudioOut};

    println!("Initializing signal chain..."); 
    assert!(init::initialize().is_ok());

    let mut mic = AudioIn::new(1);
    let mut spk = AudioOut::new(1);
    spk.inputs.set_channel(0, mic.outputs.get_channel(0));

    println!("Mirroring microphone input to speaker...");
    let mut t = 0;
    loop {
        mic.tick(t);
        spk.tick(t);
        t += 1;
    }

    // mic.stop();
    // spk.stop();
    // assert!(init::terminate().is_ok());
}
