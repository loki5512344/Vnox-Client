#[cfg(feature = "rnnoise")]
mod imp {
    use anyhow::Result;

    const FRAME_SIZE: usize = nnnoiseless::DenoiseState::FRAME_SIZE;

    pub struct DenoiseState {
        inner: Box<nnnoiseless::DenoiseState<'static>>,
        out_buf: Vec<f32>,
    }

    impl DenoiseState {
        pub fn new() -> Result<Self> {
            let inner = nnnoiseless::DenoiseState::new();
            Ok(Self {
                inner,
                out_buf: vec![0.0; FRAME_SIZE],
            })
        }

        pub fn process(&mut self, pcm: &mut [f32]) {
            for chunk in pcm.chunks_exact_mut(FRAME_SIZE) {
                let input: Vec<f32> = chunk.iter().map(|&s| s * 32768.0).collect();
                self.inner.process_frame(&mut self.out_buf, &input);
                for (out, &denoised) in chunk.iter_mut().zip(self.out_buf.iter()) {
                    *out = (denoised / 32768.0).clamp(-1.0, 1.0);
                }
            }
        }
    }
}

#[cfg(not(feature = "rnnoise"))]
mod imp {
    pub struct DenoiseState;

    impl DenoiseState {
        pub fn new() -> anyhow::Result<Self> {
            Ok(Self)
        }

        pub fn process(&mut self, _pcm: &mut [f32]) {}
    }
}

pub use imp::DenoiseState;
