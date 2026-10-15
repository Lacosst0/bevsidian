//! The reader's own sound effects ("Make it yours"): a few numbers make a sound, in the
//! spirit of sfxr: a wave, a pitch that slides, a length and a volume. Sounds are made in
//! code as WAV data, so they need no files (the web build is a single page).

use bevy::audio::AudioSource;

/// The shape of the sound wave: what the sound is made of.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Wave {
    /// Hollow and bright, the classic game beep.
    #[default]
    Square,
    /// Buzzy, for lasers and engines.
    Saw,
    /// Soft and pure.
    Sine,
    /// Hiss and crunch, for hits and explosions.
    Noise,
}

impl Wave {
    pub const ALL: [Wave; 4] = [Wave::Square, Wave::Saw, Wave::Sine, Wave::Noise];

    pub fn name(self) -> &'static str {
        match self {
            Wave::Square => "Square",
            Wave::Saw => "Saw",
            Wave::Sine => "Sine",
            Wave::Noise => "Noise",
        }
    }
}

/// A sound effect, made from a few numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sfx {
    pub wave: Wave,
    /// Where the pitch starts, in Hz.
    pub pitch: f32,
    /// Where it slides to, in octaves (-2 to 2: down to a quarter, up to four times).
    pub slide: f32,
    /// How long it lasts, in seconds.
    pub length: f32,
    /// How loud, 0 to 1.
    pub volume: f32,
}

/// The pitch, slide, length and volume a sound can have.
pub const PITCH: (f32, f32) = (60.0, 2000.0);
pub const SLIDE: (f32, f32) = (-2.0, 2.0);
pub const LENGTH: (f32, f32) = (0.04, 1.0);
pub const VOLUME: (f32, f32) = (0.05, 1.0);

impl Sfx {
    pub const fn new(wave: Wave, pitch: f32, slide: f32, length: f32, volume: f32) -> Self {
        Self {
            wave,
            pitch,
            slide,
            length,
            volume,
        }
    }

    /// The same sound with every number in range.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            wave: self.wave,
            pitch: self.pitch.clamp(PITCH.0, PITCH.1),
            slide: self.slide.clamp(SLIDE.0, SLIDE.1),
            length: self.length.clamp(LENGTH.0, LENGTH.1),
            volume: self.volume.clamp(VOLUME.0, VOLUME.1),
        }
    }

    /// As text, to store: `wave pitch slide length volume`.
    pub fn encode(&self) -> String {
        let wave = Wave::ALL.iter().position(|&w| w == self.wave).unwrap_or(0);
        format!(
            "{wave} {:.0} {:.2} {:.2} {:.2}",
            self.pitch, self.slide, self.length, self.volume
        )
    }

    pub fn decode(text: &str) -> Option<Self> {
        let mut parts = text.split_whitespace();
        let wave = *Wave::ALL.get(parts.next()?.parse::<usize>().ok()?)?;
        let mut number = || parts.next()?.parse::<f32>().ok().filter(|v| v.is_finite());
        let sfx = Sfx::new(wave, number()?, number()?, number()?, number()?);
        Some(sfx.clamped())
    }
}

/// Ready-made sounds to start from.
pub const PRESETS: [(&str, Sfx); 7] = [
    ("Coin", Sfx::new(Wave::Square, 990.0, 0.6, 0.18, 0.45)),
    ("Jump", Sfx::new(Wave::Square, 320.0, 1.0, 0.25, 0.45)),
    ("Hit", Sfx::new(Wave::Noise, 900.0, -1.2, 0.22, 0.6)),
    ("Laser", Sfx::new(Wave::Saw, 1400.0, -1.6, 0.2, 0.35)),
    ("Blip", Sfx::new(Wave::Sine, 700.0, 0.0, 0.07, 0.6)),
    ("Bounce", Sfx::new(Wave::Sine, 180.0, 1.2, 0.12, 0.7)),
    ("Boom", Sfx::new(Wave::Noise, 240.0, -1.0, 0.7, 0.7)),
];

/// The sounds the examples play, one per moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    /// Something gets hit (a rock hits the Dodge player).
    Hit,
    /// A ball bounces.
    Bounce,
    /// Something appears where you click.
    Pop,
    /// A reward.
    Coin,
}

impl Slot {
    pub const ALL: [Slot; 4] = [Slot::Hit, Slot::Bounce, Slot::Pop, Slot::Coin];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            Slot::Hit => "Hit",
            Slot::Bounce => "Bounce",
            Slot::Pop => "Pop",
            Slot::Coin => "Coin",
        }
    }

    /// Where the examples play it.
    pub fn plays(self) -> &'static str {
        match self {
            Slot::Hit => "when a rock hits you in Dodge",
            Slot::Bounce => "when a ball bounces",
            Slot::Pop => "when you place a square",
            Slot::Coin => "on the coin pad of the sound example",
        }
    }

    pub fn default_sound(self) -> Sfx {
        let preset = match self {
            Slot::Hit => "Hit",
            Slot::Bounce => "Bounce",
            Slot::Pop => "Blip",
            Slot::Coin => "Coin",
        };
        PRESETS
            .iter()
            .find(|(name, _)| *name == preset)
            .map_or(PRESETS[0].1, |(_, sfx)| *sfx)
    }
}

/// Samples per second.
pub const RATE: u32 = 22_050;

/// The sound's samples, -1 to 1.
pub fn samples(sfx: &Sfx) -> Vec<f32> {
    let sfx = sfx.clamped();
    let count = (sfx.length * RATE as f32) as usize;
    let attack = (0.005 * RATE as f32).max(1.0);
    let mut phase = 0.0f32;
    let mut noise = 0.0f32;
    // A small, fixed random generator: the same numbers always make the same sound.
    let mut seed = 0x2545_f491_u32;
    let mut random = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed as f32 / u32::MAX as f32 * 2.0 - 1.0
    };
    (0..count)
        .map(|i| {
            let t = i as f32 / count.max(1) as f32;
            // The pitch slides smoothly, in octaves.
            let pitch = sfx.pitch * (sfx.slide * t).exp2();
            let before = phase;
            phase = (phase + pitch / RATE as f32).fract();
            let value = match sfx.wave {
                Wave::Square => {
                    if phase < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Wave::Saw => phase * 2.0 - 1.0,
                Wave::Sine => (phase * std::f32::consts::TAU).sin(),
                Wave::Noise => {
                    // A new random value every cycle: higher pitch, finer hiss.
                    if phase < before {
                        noise = random();
                    }
                    noise
                }
            };
            // A quick start (no click), then fading to nothing.
            let envelope = (i as f32 / attack).min(1.0) * (1.0 - t).powf(1.5);
            value * envelope * sfx.volume * 0.6
        })
        .collect()
}

/// The sound as a WAV file (16-bit mono).
pub fn wav(sfx: &Sfx) -> Vec<u8> {
    let samples = samples(sfx);
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}

pub fn audio(sfx: &Sfx) -> AudioSource {
    AudioSource {
        bytes: wav(sfx).into(),
    }
}

/// A new sound from `seed`: a surprise to start from.
pub fn random(seed: u32) -> Sfx {
    let mut state = seed.max(1);
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state as f32 / u32::MAX as f32
    };
    let wave = Wave::ALL[(next() * 4.0) as usize % 4];
    let lerp = |(lo, hi): (f32, f32), t: f32| lo + (hi - lo) * t;
    Sfx::new(
        wave,
        lerp(PITCH, next() * next()),
        lerp(SLIDE, next()),
        lerp(LENGTH, next() * 0.6),
        lerp((0.3, 0.8), next()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sound_is_a_wav_file_as_long_as_asked() {
        let sfx = Sfx::new(Wave::Square, 440.0, 0.0, 0.5, 0.5);
        let bytes = wav(&sfx);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..16], b"WAVEfmt ");
        let samples = (bytes.len() - 44) / 2;
        assert_eq!(samples, (0.5 * RATE as f32) as usize);
        // It starts and ends quietly (no clicks) and is loud in between.
        let all = super::samples(&sfx);
        assert!(all[0].abs() < 0.01 && all.last().unwrap().abs() < 0.01);
        assert!(all.iter().any(|s| s.abs() > 0.2));
    }

    #[test]
    fn sounds_survive_being_stored() {
        for (_, sfx) in PRESETS {
            let back = Sfx::decode(&sfx.encode()).unwrap();
            assert_eq!(back.wave, sfx.wave);
            assert!((back.pitch - sfx.pitch).abs() < 1.0);
            assert!((back.length - sfx.length).abs() < 0.01);
        }
        assert!(Sfx::decode("9 1 1 1 1").is_none(), "no such wave");
        assert!(Sfx::decode("0 nan 0 0.1 0.5").is_none());
        // Out of range numbers come back in range.
        let wild = Sfx::decode("0 99999 9 9 9").unwrap();
        assert_eq!(wild, wild.clamped());
    }

    #[test]
    fn every_wave_makes_a_sound() {
        for wave in Wave::ALL {
            let all = samples(&Sfx::new(wave, 300.0, 0.5, 0.2, 0.8));
            assert!(all.iter().any(|s| s.abs() > 0.1), "{wave:?}");
        }
        for seed in 1..20 {
            assert_eq!(random(seed), random(seed).clamped());
        }
    }
}
