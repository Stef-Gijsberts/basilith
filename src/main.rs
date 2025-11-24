use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{self, FromSample, SizedSample};
use dasp::{Frame, Sample, Signal, ring_buffer, signal};

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

fn run<T: SizedSample + FromSample<f64>>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
) -> Result<(), anyhow::Error> {
    let a_4 = signal::rate(config.sample_rate.0.into()).const_hz(440.0);

    let mut sine = a_4.sine();

    // Create and run the stream.
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let channels = config.channels as usize;
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| write_data(data, channels, &mut sine),
        err_fn,
        None,
    )?;
    stream.play()?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, signal: &mut dyn Signal<Frame = f64>)
where
    T: Sample + FromSample<f64>,
{
    for frame in output.chunks_mut(channels) {
        for frame_sample in frame.iter_mut() {
            *frame_sample = signal.next().to_sample::<T>();
        }
    }
}
