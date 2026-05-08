use crate::Result;
use opus_rs::{Application, OpusDecoder, OpusEncoder};

const QUALITY: QualityPreset = QualityPreset::High;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum QualityPreset {
    Low,
    Middle,
    High,
}

impl QualityPreset {
    const fn sample_rate(self) -> i32 {
        match self {
            QualityPreset::Low => 16000,
            QualityPreset::Middle => 24000,
            QualityPreset::High => 48000,
        }
    }

    const fn frame_ms(self) -> usize {
        match self {
            QualityPreset::Low => 10,
            QualityPreset::Middle => 10,
            QualityPreset::High => 10,
        }
    }

    const fn bitrate_bps(self) -> i32 {
        match self {
            QualityPreset::Low => 16000,
            QualityPreset::Middle => 24000,
            QualityPreset::High => 64000,
        }
    }

    const fn application(self) -> Application {
        match self {
            QualityPreset::Low => Application::RestrictedLowDelay,
            QualityPreset::Middle => Application::Voip,
            QualityPreset::High => Application::Audio,
        }
    }

    const fn use_cbr(self) -> bool {
        match self {
            QualityPreset::Low => true,
            QualityPreset::Middle => true,
            QualityPreset::High => false,
        }
    }
}

pub const SAMPLE_RATE: i32 = QUALITY.sample_rate();
const FRAME_MS: usize = QUALITY.frame_ms();
const BITRATE_BPS: i32 = QUALITY.bitrate_bps();
const USE_CBR: bool = QUALITY.use_cbr();
const CHANNELS: usize = 1;

pub const SAMPLES_PER_FRAME: usize = SAMPLE_RATE as usize * FRAME_MS / 1000;

pub struct Codec {
    encoder: OpusEncoder,
    decoder: OpusDecoder,
    encode_buffer: Vec<u8>,
    decode_buffer: Vec<f32>,
}

impl Codec {
    pub fn new() -> Result<Self> {
        let mut encoder = OpusEncoder::new(SAMPLE_RATE, CHANNELS, QUALITY.application())?;
        encoder.bitrate_bps = BITRATE_BPS;
        encoder.use_cbr = USE_CBR;

        let decoder = OpusDecoder::new(SAMPLE_RATE, CHANNELS)?;

        Ok(Self {
            encoder,
            decoder,
            encode_buffer: vec![0u8; 512],
            decode_buffer: vec![0.0f32; SAMPLES_PER_FRAME],
        })
    }

    pub fn encode(&mut self, pcm: &[f32]) -> Result<&[u8]> {
        let samples = pcm.len().min(SAMPLES_PER_FRAME);
        let len = self.encoder.encode(pcm, samples, &mut self.encode_buffer)?;
        Ok(&self.encode_buffer[..len])
    }

    pub fn decode(&mut self, data: &[u8]) -> Result<&[f32]> {
        let samples = self
            .decoder
            .decode(data, SAMPLES_PER_FRAME, &mut self.decode_buffer)?;
        Ok(&self.decode_buffer[..samples])
    }

    pub fn config_summary() -> String {
        format!(
            "{:?}: {} Hz, {}ch, {}ms, {} kbps",
            QUALITY,
            SAMPLE_RATE,
            CHANNELS,
            FRAME_MS,
            BITRATE_BPS / 1000,
            // if USE_CBR { "CBR" } else { "VBR" }
        )
    }
}
