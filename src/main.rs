use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{self, SizedSample};
use dasp::{Sample, Signal, signal};
use std::sync::mpsc;

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config()?;

    match config.sample_format() {
        cpal::SampleFormat::F64 => run::<f64>(&device, &config.into())?,
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into())?,
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }

    Ok(())
}

fn run<T: SizedSample + From<f32>>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
) -> Result<(), anyhow::Error> {
    // Create a signal chain to play back 1 second of each oscillator at A4.
    let hz = signal::rate(config.sample_rate.0 as f64).const_hz(440.0);
    let one_sec = config.sample_rate.0 as usize;
    let mut synth = hz
        .clone()
        .sine()
        .take(one_sec)
        .chain(hz.clone().saw().take(one_sec))
        .chain(hz.clone().square().take(one_sec))
        .chain(hz.clone().noise_simplex().take(one_sec))
        .chain(signal::noise(0).take(one_sec))
        .map(|s| s.to_sample::<f32>() * 0.2);

    // A channel for indicating when playback has completed.
    let (complete_tx, complete_rx) = mpsc::sync_channel(1);

    // Create and run the stream.
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let channels = config.channels as usize;
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &complete_tx, &mut synth)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    // Wait for playback to complete.
    complete_rx.recv().unwrap();
    stream.pause()?;

    Ok(())
}

fn write_data<T: SizedSample + From<f32>>(
    output: &mut [T],
    channels: usize,
    complete_tx: &mpsc::SyncSender<()>,
    signal: &mut dyn Iterator<Item = f32>,
) {
    for frame in output.chunks_mut(channels) {
        let sample = match signal.next() {
            None => {
                complete_tx.try_send(()).ok();
                0.0
            }
            Some(sample) => sample,
        };
        let value: T = sample.into();
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
