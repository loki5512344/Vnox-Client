mod rnnoise;

pub use rnnoise::DenoiseState;

pub struct TransmitGate {
    pub mic_enabled: bool,
    pub vad_mode: u8,
    pub vad_threshold: f32,
    pub ptt_held: bool,
    pub input_volume: f32,
    pub noise_suppress: bool,
}

impl Default for TransmitGate {
    fn default() -> Self {
        Self {
            mic_enabled: true,
            vad_mode: 0,
            vad_threshold: 40.0,
            ptt_held: false,
            input_volume: 80.0,
            noise_suppress: false,
        }
    }
}

impl Clone for TransmitGate {
    fn clone(&self) -> Self {
        Self {
            mic_enabled: self.mic_enabled,
            vad_mode: self.vad_mode,
            vad_threshold: self.vad_threshold,
            ptt_held: self.ptt_held,
            input_volume: self.input_volume,
            noise_suppress: self.noise_suppress,
        }
    }
}

impl std::fmt::Debug for TransmitGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransmitGate")
            .field("mic_enabled", &self.mic_enabled)
            .field("vad_mode", &self.vad_mode)
            .field("vad_threshold", &self.vad_threshold)
            .field("ptt_held", &self.ptt_held)
            .field("input_volume", &self.input_volume)
            .field("noise_suppress", &self.noise_suppress)
            .finish()
    }
}

pub fn frame_rms(pcm: &[f32]) -> f32 {
    if pcm.is_empty() {
        return 0.0;
    }
    let mean_sq: f32 = pcm.iter().map(|s| s * s).sum::<f32>() / pcm.len() as f32;
    mean_sq.sqrt()
}

fn vad_rms_threshold(ui_threshold: f32) -> f32 {
    0.002 + (ui_threshold.clamp(0.0, 100.0) / 100.0) * 0.118
}

impl TransmitGate {
    pub fn apply_input_volume(pcm: &mut [f32], volume_pct: f32) {
        let gain = (volume_pct / 100.0).clamp(0.0, 2.0);
        if (gain - 1.0).abs() < f32::EPSILON {
            return;
        }
        for s in pcm {
            *s *= gain;
        }
    }

    pub fn should_transmit(&self, pcm: &[f32]) -> bool {
        if !self.mic_enabled {
            return false;
        }
        match self.vad_mode {
            2 => true,
            0 => self.ptt_held,
            1 => frame_rms(pcm) >= vad_rms_threshold(self.vad_threshold),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loud_frame() -> Vec<f32> {
        vec![0.25f32; 960]
    }

    fn quiet_frame() -> Vec<f32> {
        vec![0.0001f32; 960]
    }

    #[test]
    fn mic_disabled_blocks() {
        let gate = TransmitGate {
            mic_enabled: false,
            vad_mode: 2,
            ..Default::default()
        };
        assert!(!gate.should_transmit(&loud_frame()));
    }

    #[test]
    fn always_on_when_mic_enabled() {
        let gate = TransmitGate {
            vad_mode: 2,
            ..Default::default()
        };
        assert!(gate.should_transmit(&quiet_frame()));
    }

    #[test]
    fn ptt_requires_held() {
        let mut gate = TransmitGate {
            vad_mode: 0,
            ptt_held: false,
            ..Default::default()
        };
        assert!(!gate.should_transmit(&loud_frame()));
        gate.ptt_held = true;
        assert!(gate.should_transmit(&loud_frame()));
    }

    #[test]
    fn vad_uses_threshold() {
        let gate = TransmitGate {
            vad_mode: 1,
            vad_threshold: 40.0,
            ..Default::default()
        };
        assert!(!gate.should_transmit(&quiet_frame()));
        assert!(gate.should_transmit(&loud_frame()));
    }
}
