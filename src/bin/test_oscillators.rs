//! Writes wav files for every Oscillator waveform for analysis

extern crate oxcable;

#[cfg(not(test))]
fn main() {
    use oxcable::chain::DeviceChain;
    use oxcable::io::wav::WavWriter;
    use oxcable::oscillator::*;
    use oxcable::utils::tick::tick_n_times;

    // Initialize objects
    println!("Initializing signal chain...");
    let freq = 8000.0;
    let mut chains: Vec<DeviceChain> = Vec::new();
    chains.push(DeviceChain::from(Oscillator::new(Sine).freq(freq))
        .into(WavWriter::new("wav/test_sine.wav", 1)));
    chains.push(DeviceChain::from(Oscillator::new(Saw(Aliased)).freq(freq))
        .into(WavWriter::new("wav/test_saw_naive.wav", 1)));
    chains.push(DeviceChain::from(Oscillator::new(Saw(PolyBlep)).freq(freq))
        .into(WavWriter::new("wav/test_saw_blep.wav", 1)));
    chains.push(DeviceChain::from(Oscillator::new(Square(Aliased)).freq(freq))
        .into(WavWriter::new("wav/test_square_naive.wav", 1)));
    chains.push(DeviceChain::from(Oscillator::new(Square(PolyBlep)).freq(freq))
        .into(WavWriter::new("wav/test_square_blep.wav", 1)));
    chains.push(DeviceChain::from(Oscillator::new(Tri(Aliased)).freq(freq))
        .into(WavWriter::new("wav/test_tri_naive.wav", 1)));
    chains.push(DeviceChain::from(Oscillator::new(Tri(PolyBlep)).freq(freq))
        .into(WavWriter::new("wav/test_tri_blep.wav", 1)));
    chains.push(DeviceChain::from(Oscillator::new(WhiteNoise).freq(freq))
        .into(WavWriter::new("wav/test_white_noise.wav", 1)));
    chains.push(DeviceChain::from(Oscillator::new(PulseTrain).freq(freq))
        .into(WavWriter::new("wav/test_pulse_train.wav", 1)));

    // Write files
    println!("Writing oscillators to wav files...");
    for chain in chains.iter_mut() {
        tick_n_times(chain, 44100);
    }
    println!("Done");
}
