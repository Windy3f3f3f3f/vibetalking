use crate::config::SAMPLE_RATE;
use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

pub struct Recorder {
    inner: Arc<Mutex<Option<Active>>>,
}

struct Active {
    _stream: Stream,
    buffer: Arc<Mutex<Vec<i16>>>,
    source_rate: u32,
    source_channels: u16,
    started: Instant,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self) -> Result<()> {
        let mut guard = self.inner.lock();
        if guard.is_some() {
            return Ok(());
        }
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow!("no default input device"))?;
        let default_config = device.default_input_config()?;
        let source_rate = default_config.sample_rate().0;
        let source_channels = default_config.channels();
        let sample_format = default_config.sample_format();
        let config: StreamConfig = default_config.into();

        let buffer = Arc::new(Mutex::new(Vec::<i16>::with_capacity(
            (source_rate as usize) * 30,
        )));
        let buf_cb = buffer.clone();
        let err_fn = |e| log::error!("cpal stream error: {}", e);

        let stream = match sample_format {
            SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let mut b = buf_cb.lock();
                    b.extend(data.iter().map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16));
                },
                err_fn,
                None,
            )?,
            SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    let mut b = buf_cb.lock();
                    b.extend_from_slice(data);
                },
                err_fn,
                None,
            )?,
            SampleFormat::U16 => device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    let mut b = buf_cb.lock();
                    b.extend(data.iter().map(|&s| (s as i32 - 32768) as i16));
                },
                err_fn,
                None,
            )?,
            fmt => return Err(anyhow!("unsupported sample format: {:?}", fmt)),
        };
        stream.play()?;

        *guard = Some(Active {
            _stream: stream,
            buffer,
            source_rate,
            source_channels,
            started: Instant::now(),
        });
        Ok(())
    }

    pub fn stop(&self) -> Result<(Vec<u8>, u64)> {
        let active = self
            .inner
            .lock()
            .take()
            .ok_or_else(|| anyhow!("recorder not running"))?;
        let duration_ms = active.started.elapsed().as_millis() as u64;
        let raw = active.buffer.lock().clone();

        let mono = if active.source_channels > 1 {
            downmix(&raw, active.source_channels as usize)
        } else {
            raw
        };
        let resampled = if active.source_rate != SAMPLE_RATE {
            resample_linear(&mono, active.source_rate, SAMPLE_RATE)
        } else {
            mono
        };
        let wav = build_wav(&resampled, SAMPLE_RATE);
        Ok((wav, duration_ms))
    }

    pub fn is_running(&self) -> bool {
        self.inner.lock().is_some()
    }
}

fn downmix(samples: &[i16], channels: usize) -> Vec<i16> {
    samples
        .chunks_exact(channels)
        .map(|ch| {
            let sum: i32 = ch.iter().map(|&s| s as i32).sum();
            (sum / channels as i32) as i16
        })
        .collect()
}

fn resample_linear(input: &[i16], src_rate: u32, dst_rate: u32) -> Vec<i16> {
    if input.is_empty() {
        return Vec::new();
    }
    let ratio = dst_rate as f64 / src_rate as f64;
    let out_len = (input.len() as f64 * ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    let last_idx = input.len() - 1;
    for i in 0..out_len {
        let src_idx = i as f64 / ratio;
        let idx = src_idx.floor() as usize;
        let frac = src_idx - idx as f64;
        let a = input[idx.min(last_idx)] as f64;
        let b = input[(idx + 1).min(last_idx)] as f64;
        out.push((a + (b - a) * frac) as i16);
    }
    out
}

/// Build a PCM WAV (16-bit little-endian mono) with 44-byte RIFF header.
fn build_wav(pcm: &[i16], sample_rate: u32) -> Vec<u8> {
    let channels: u16 = 1;
    let bits: u16 = 16;
    let byte_rate = sample_rate * (channels as u32) * (bits as u32) / 8;
    let block_align = channels * bits / 8;
    let data_len = (pcm.len() * 2) as u32;
    let riff_len = 36 + data_len;

    let mut out = Vec::with_capacity(44 + pcm.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_len.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // PCM subchunk size
    out.extend_from_slice(&1u16.to_le_bytes()); // format: PCM
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&bits.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for &s in pcm {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}
