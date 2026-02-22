use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::{mpsc::Receiver, Arc, Mutex};
use std::time::Duration;

fn write_f32_input(
    input: &[f32],
    channels: usize,
    writer: &Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>,
) {
    if let Ok(mut guard) = writer.lock() {
        if let Some(wav_writer) = guard.as_mut() {
            for frame in input.chunks(channels) {
                let sample = frame[0].clamp(-1.0, 1.0);
                let _ = wav_writer.write_sample((sample * i16::MAX as f32) as i16);
            }
        }
    }
}

fn write_i16_input(
    input: &[i16],
    channels: usize,
    writer: &Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>,
) {
    if let Ok(mut guard) = writer.lock() {
        if let Some(wav_writer) = guard.as_mut() {
            for frame in input.chunks(channels) {
                let _ = wav_writer.write_sample(frame[0]);
            }
        }
    }
}

fn write_u16_input(
    input: &[u16],
    channels: usize,
    writer: &Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>,
) {
    if let Ok(mut guard) = writer.lock() {
        if let Some(wav_writer) = guard.as_mut() {
            for frame in input.chunks(channels) {
                let normalized = frame[0] as i32 - i16::MAX as i32;
                let _ = wav_writer.write_sample(normalized as i16);
            }
        }
    }
}

pub fn record_until_stopped(audio_path: PathBuf, stop_rx: Receiver<()>) -> Result<(), String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "no input device found".to_string())?;

    let supported_config = device
        .default_input_config()
        .map_err(|e| format!("failed to get input config: {e}"))?;

    let channels = supported_config.channels() as usize;
    let sample_rate = supported_config.sample_rate().0;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let writer = hound::WavWriter::create(audio_path, spec)
        .map_err(|e| format!("failed to create wav writer: {e}"))?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    let err_fn = |err| eprintln!("audio stream error: {err}");

    let stream_config: cpal::StreamConfig = supported_config.config();
    let stream = match supported_config.sample_format() {
        cpal::SampleFormat::F32 => {
            let writer = Arc::clone(&writer);
            device
                .build_input_stream(
                    &stream_config,
                    move |data: &[f32], _| write_f32_input(data, channels, &writer),
                    err_fn,
                    None,
                )
                .map_err(|e| format!("failed to build f32 stream: {e}"))?
        }
        cpal::SampleFormat::I16 => {
            let writer = Arc::clone(&writer);
            device
                .build_input_stream(
                    &stream_config,
                    move |data: &[i16], _| write_i16_input(data, channels, &writer),
                    err_fn,
                    None,
                )
                .map_err(|e| format!("failed to build i16 stream: {e}"))?
        }
        cpal::SampleFormat::U16 => {
            let writer = Arc::clone(&writer);
            device
                .build_input_stream(
                    &stream_config,
                    move |data: &[u16], _| write_u16_input(data, channels, &writer),
                    err_fn,
                    None,
                )
                .map_err(|e| format!("failed to build u16 stream: {e}"))?
        }
        other => {
            return Err(format!("unsupported sample format: {other:?}"));
        }
    };

    stream
        .play()
        .map_err(|e| format!("failed to start input stream: {e}"))?;

    loop {
        if stop_rx.recv_timeout(Duration::from_millis(150)).is_ok() {
            break;
        }
    }

    drop(stream);
    if let Ok(mut guard) = writer.lock() {
        if let Some(wav_writer) = guard.take() {
            wav_writer
                .finalize()
                .map_err(|e| format!("failed to finalize wav: {e}"))?;
        }
    }

    Ok(())
}
