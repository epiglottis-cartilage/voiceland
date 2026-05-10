use crate::Result;
use nnnoiseless::DenoiseState;
use opus_rs::{Application, OpusDecoder, OpusEncoder};
use std::mem::MaybeUninit;

pub const SAMPLE_RATE: i32 = 48000;
pub const FRAME_MS: usize = 10;
pub const BITRATE_BPS: i32 = 512000;
const USE_CBR: bool = false;
const CHANNELS: usize = 1;
const APPLICATION: Application = Application::Audio;

pub const SAMPLES_PER_FRAME: usize = SAMPLE_RATE as usize * FRAME_MS / 1000;
// assert_eq!(SAMPLES_PER_FRAME, 480);

pub struct Encoder {
    encoder: OpusEncoder,
    encode_buffer: Box<[u8; 1500]>,
    denoiser: Option<Box<DenoiseState<'static>>>,
}

impl Encoder {
    pub fn new(desoise: bool) -> Result<Self> {
        let mut encoder = OpusEncoder::new(SAMPLE_RATE, CHANNELS, APPLICATION)?;
        encoder.bitrate_bps = BITRATE_BPS;
        encoder.use_cbr = USE_CBR;
        let denoiser = if desoise {
            Some(DenoiseState::new())
        } else {
            None
        };

        Ok(Self {
            encoder,
            encode_buffer: unsafe { Box::new_uninit().assume_init() },
            denoiser,
        })
    }

    pub fn encode(&mut self, pcm: &mut [f32; SAMPLES_PER_FRAME]) -> Result<&[u8]> {
        let len;
        if let Some(denoiser) = &mut self.denoiser {
            let mut denoised_pcm = unsafe {
                MaybeUninit::array_assume_init([MaybeUninit::uninit(); SAMPLES_PER_FRAME])
            };
            pcm.iter_mut().for_each(|x| *x *= 32768.);
            denoiser.process_frame(&mut denoised_pcm, pcm);
            denoised_pcm.iter_mut().for_each(|x| *x /= 32768.);
            len = self.encoder.encode(
                &mut denoised_pcm,
                SAMPLES_PER_FRAME,
                &mut *self.encode_buffer,
            )?;
        } else {
            len = self
                .encoder
                .encode(pcm, SAMPLES_PER_FRAME, &mut *self.encode_buffer)?;
        }
        Ok(&self.encode_buffer[..len])
    }
}
pub struct Decoder {
    decoder: OpusDecoder,
    decode_buffer: Box<[f32; SAMPLES_PER_FRAME]>,
}

impl Decoder {
    pub fn new() -> Result<Self> {
        let decoder = OpusDecoder::new(SAMPLE_RATE, CHANNELS)?;

        Ok(Self {
            decoder,
            decode_buffer: unsafe { Box::new_uninit().assume_init() },
        })
    }

    pub fn decode(&mut self, data: &[u8]) -> Result<&[f32; SAMPLES_PER_FRAME]> {
        let samples = self
            .decoder
            .decode(data, SAMPLES_PER_FRAME, &mut *self.decode_buffer)?;
        assert_eq!(samples, SAMPLES_PER_FRAME);
        Ok(&*self.decode_buffer)
    }
}

pub fn config_summary() -> String {
    format!(
        "{} Hz, {}ch, {}ms, {} kbps",
        SAMPLE_RATE,
        CHANNELS,
        FRAME_MS,
        BITRATE_BPS / 1000,
    )
}
