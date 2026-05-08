use crate::{Result, app::App, codec::Codec};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{
    collections::VecDeque,
    iter::repeat,
    sync::{Arc, Mutex, atomic::AtomicU16},
    time::Duration,
};
use tokio::sync::mpsc;

use crate::codec::{SAMPLE_RATE as CODEC_SAMPLE_RATE, SAMPLES_PER_FRAME as CODEC_SAMPLES};

const SAMPLE_RATE: u32 = CODEC_SAMPLE_RATE as u32;
const CHANNELS: u16 = 1;
const FRAME_MS: u64 = 10;
const SAMPLES_PER_FRAME: usize = CODEC_SAMPLES;

pub struct AudioApp {
    _input_stream: cpal::Stream,
    _output_stream: cpal::Stream,
    playback_buffer: Arc<Mutex<VecDeque<f32>>>,
    codec: Arc<Mutex<Codec>>,
}

impl AudioApp {
    pub async fn new(
        log_tx: mpsc::Sender<String>,
        record_tx: mpsc::Sender<Vec<u8>>,
        volume: Arc<AtomicU16>,
    ) -> Result<Self> {
        let codec = Arc::new(Mutex::new(Codec::new()?));
        let host = cpal::default_host();

        // --- Input (microphone) ---
        let input_device = host
            .default_input_device()
            .ok_or("No default input device")?;
        let input_config = cpal::StreamConfig {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE.into(),
            buffer_size: cpal::BufferSize::Default,
        };

        let log_err = log_tx.clone();
        let mut sample_buffer: Vec<f32> = Vec::new();
        let codec_enc = codec.clone();

        let input_stream = input_device.build_input_stream(
            &input_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                sample_buffer.extend_from_slice(data);
                while sample_buffer.len() >= SAMPLES_PER_FRAME {
                    let mut frame: Vec<f32> = sample_buffer.drain(..SAMPLES_PER_FRAME).collect();

                    let v = volume.load(std::sync::atomic::Ordering::Relaxed) as f32 / 100.;
                    frame.iter_mut().for_each(|s| *s *= v);

                    if let Ok(mut c) = codec_enc.lock() {
                        if let Ok(encoded) = c.encode(&frame) {
                            let _ = record_tx.try_send(encoded.to_vec());
                        }
                    }
                }
            },
            move |err| {
                let _ = log_err.try_send(format!("Audio input error: {}", err));
            },
            None,
        )?;

        // --- Output (speaker) ---
        let output_device = host
            .default_output_device()
            .ok_or("No default output device")?;
        let output_config = cpal::StreamConfig {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE.into(),
            buffer_size: cpal::BufferSize::Default,
        };

        let playback_buffer: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::new()));
        let playback_buffer_clone = playback_buffer.clone();

        let log_err = log_tx.clone();
        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut pb = playback_buffer_clone.lock().unwrap();
                let len = data.len().min(pb.len());
                data.iter_mut()
                    .zip(pb.drain(..len).chain(repeat(0.0)))
                    .for_each(|(d, p)| *d = p);
            },
            move |err| {
                let _ = log_err.try_send(format!("Audio output error: {}", err));
            },
            None,
        )?;

        input_stream.play()?;
        output_stream.play()?;
        log_tx
            .send(format!("Audio initialized ({})", Codec::config_summary()))
            .await?;

        Ok(Self {
            _input_stream: input_stream,
            _output_stream: output_stream,
            playback_buffer,
            codec,
        })
    }

    pub async fn run(&mut self, app: &App) {
        let mut interval = tokio::time::interval(Duration::from_millis(FRAME_MS));

        loop {
            if !app.running.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            interval.tick().await;

            let mut mixed = vec![0.0f32; SAMPLES_PER_FRAME];
            let mut has_data = false;

            let peers: tokio::sync::RwLockReadGuard<'_, Vec<crate::peer::Peer>> =
                app.peers.read().await;
            for peer in peers.iter() {
                if let Some(frame_bytes) = peer.try_pop_voice() {
                    has_data = true;
                    let volume =
                        peer.volume.load(std::sync::atomic::Ordering::Relaxed) as f32 / 255.0;

                    if let Ok(mut c) = self.codec.lock() {
                        if let Ok(decoded) = c.decode(&frame_bytes) {
                            for (i, sample) in decoded.iter().enumerate().take(SAMPLES_PER_FRAME) {
                                mixed[i] += sample * volume;
                            }
                        }
                    }
                }
            }
            drop(peers);

            if has_data {
                let mut pb = self.playback_buffer.lock().unwrap();
                pb.extend(&mixed);
            }
        }
    }
}
