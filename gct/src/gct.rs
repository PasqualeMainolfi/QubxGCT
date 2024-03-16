use rand::{rngs::ThreadRng, Rng};
use std::sync::Arc;

#[derive(Debug)]
pub enum TableMode {
    Sine,
}

pub struct Table {} 

impl Table {

    pub fn table(mode: TableMode, length: usize) -> Vec<f32>{
        let mut t = vec![0.0; length];
        
        match mode {
            TableMode::Sine => {
                for (i, sample) in t.iter_mut().enumerate() {
                    *sample = (2.0 * std::f32::consts::PI * i as f32 / (length as f32 - 1.0)).sin()
                }
                t
            }
        }
    }

}

#[derive(Debug, Clone)]
pub enum EnvelopeMode {
    Rect,
    Percussive,
    Hanning, 
}

pub struct Envelope {}

impl Envelope {
    pub fn generate_envelope(env_mode: EnvelopeMode, length: usize) -> Vec<f32> {

        match env_mode {
            EnvelopeMode::Rect => {
                let envelope: Vec<f32> = vec![1.0; length];

                envelope
            },

            EnvelopeMode::Hanning => {
                let mut envelope: Vec<f32> = vec![0.0; length];
                envelope.iter_mut().enumerate().for_each(|(i, x)| {
                    *x = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (length - 1) as f32).cos());  
                });

                envelope
            },

            EnvelopeMode::Percussive => {
                let mut envelope: Vec<f32> = vec![0.0; length];

                let mut n: i32 = (length as i32 / 50) + 1;
                if n <= 1 { n = 2 };

                let step_up = 1.0 / (n as f32 - 1.0);
                let step_down  = 1.0 / (length as f32 - n as f32 - 1.0);

                envelope.iter_mut().enumerate().for_each(|(i, x)| {
                    *x = if i < n as usize { step_up * i as f32 } else { 1.0 - (step_down * (i as f32 - n as f32)) };
                }); 

                envelope
            }
        }

    }
}

#[derive(Debug)]
pub struct ParamRange {
    pub min: f32,
    pub max: f32
}

impl ParamRange {
    
}

#[derive(Debug)]
pub enum ReverseMode {
    Reverse,
    NotReverse,
    RandomReverse
}

#[derive(Debug)]
pub struct GrainSettings {
    pub dur_range: ParamRange,
    pub freq_range: ParamRange,
    pub reverse: ReverseMode,
    pub pan_range: ParamRange,
    pub amp_range: ParamRange,
    pub time_pos_range: ParamRange,
    pub delay_range: ParamRange,
    pub sr: i32,
    pub env_mode: Option<EnvelopeMode>
}

impl Default for GrainSettings {
    fn default() -> Self {
        Self {

            dur_range: ParamRange { min: 0.1, max: 1.0, },
            freq_range: ParamRange { min: 1.0, max: 1.0 },
            reverse: ReverseMode::NotReverse,
            pan_range: ParamRange { min: 0.5, max: 0.5 },
            amp_range: ParamRange{ min: 0.707, max: 0.707 },
            time_pos_range: ParamRange { min: 0.0, max: 0.0 },
            delay_range: ParamRange { min: 0.1, max: 0.1 },
            sr: 44100,
            env_mode: None

        }
    }
}

pub struct GrainParams {

    pub dur: i32,
    pub freq: f32,
    pub rev: bool,
    pub pan_pos: f32,
    pub amp: f32,
    pub env_mode: Option<EnvelopeMode>,
    pub time_pos: f32,
    pub delay: f32,
    grain_settings: Arc<GrainSettings>,
    rnd: ThreadRng

}

impl GrainParams {

    pub fn new(grain_settings: Arc<GrainSettings>) -> Self {
        let mut rnd = rand::thread_rng();

        let dur: i32 = (Self::get_random_value(&mut rnd, &grain_settings.dur_range) * grain_settings.sr as f32) as i32;
        let time_pos: f32 = Self::get_random_value(&mut rnd, &grain_settings.time_pos_range);
        let delay: f32 = Self::get_random_value(&mut rnd, &grain_settings.delay_range);
        let freq: f32 = Self::get_random_value(&mut rnd,&grain_settings.freq_range);
        let pan_pos : f32 = Self::get_random_value(&mut rnd, &grain_settings.pan_range);
        let amp: f32 = Self::get_random_value(&mut rnd, &grain_settings.amp_range);
        let env_mode = grain_settings.env_mode.clone();

        let rev = match grain_settings.reverse {
            ReverseMode::NotReverse => false,
            ReverseMode::Reverse => true,
            ReverseMode::RandomReverse => rnd.gen_bool(0.5) 
        };

        Self {

            dur,
            freq,
            rev,
            pan_pos,
            amp,
            env_mode,
            time_pos,
            delay,
            grain_settings,
            rnd

        }

    }

    pub fn update_params(&mut self) {

        self.dur = (Self::get_random_value(&mut self.rnd, &self.grain_settings.dur_range) * self.grain_settings.sr as f32) as i32;
        self.time_pos = Self::get_random_value(&mut self.rnd, &self.grain_settings.time_pos_range);
        self.delay = Self::get_random_value(&mut self.rnd, &self.grain_settings.delay_range);
        self.freq = Self::get_random_value(&mut self.rnd, &self.grain_settings.freq_range);
        self.pan_pos = Self::get_random_value(&mut self.rnd, &self.grain_settings.pan_range);
        self.amp = Self::get_random_value(&mut self.rnd, &self.grain_settings.amp_range);
        self.rev = match self.grain_settings.reverse {
            ReverseMode::NotReverse => false,
            ReverseMode::Reverse => true,
            ReverseMode::RandomReverse => self.rnd.gen_bool(0.5) 
        };

    }

    fn get_random_value(rnd: &mut ThreadRng, range: &ParamRange) -> f32 {
        let value: f32 = if range.min != range.max {
            rnd.gen_range(range.min..range.max)
        } else {
            range.min
        };
        value
    }
}

unsafe impl Sync for GrainParams {} 
unsafe impl Send for GrainParams {}

pub struct Grain {

    grain_size: i32,
    amp: f32,
    env: Vec<f32>,
    left_pan_value: f32,
    right_pan_value: f32

}

impl Grain {
    
    pub fn new(grain_size: i32, amp: f32, pan: f32, env_mode: Option<EnvelopeMode>) -> Self { 

        let env = match env_mode {
            Some(envelope) => Envelope::generate_envelope(envelope, grain_size as usize),
            None => Envelope::generate_envelope(EnvelopeMode::Rect, grain_size as usize)
        };

        Self { 

            grain_size, 
            amp, 
            env, 
            left_pan_value: pan.sqrt(),
            right_pan_value: (1.0 - pan).sqrt()
        
        }
    }

    pub fn generate_sound_grain(&self, sig: &[f32], time_pos: f32, speed: f32, reverse: bool) -> Vec<f32> {

        let sig_size = sig.len();
        let mut grain = vec![0.0; self.grain_size as usize];
        let mut phase = if time_pos >= 0.0 { time_pos * sig_size as f32 } else { 0.0 };
        phase %= sig_size as f32;
        
        for (i, sample) in grain.iter_mut().enumerate() {

            
            let mut increment: f32 = i as f32 * speed;
            
            if increment >= self.grain_size as f32 - 1.0 { increment -= self.grain_size as f32 }
            
            let intndx: f32 = increment.floor();
            let fracndx = increment - intndx;
            
            let mut prev_index = (phase + intndx) as i32 % (sig_size as i32 - 1);
            if prev_index < 0 { prev_index = 0 }
            let next_index = prev_index + 1;

            // println!("[DEBUG] index in gct riga 241 {} {} {}", i, prev_index, next_index);
            
            *sample = ((1.0 - fracndx) * sig[prev_index as usize] + fracndx * sig[next_index as usize]) * self.env[i] * self.amp;
            
        }
        
        let mut interleaved_grain = vec![0.0; self.grain_size as usize * 2];
        for i in 0..self.grain_size as usize {
            interleaved_grain[i * 2] = grain[i] * self.left_pan_value;
            interleaved_grain[i * 2 + 1] = grain[i] * self.right_pan_value;
        }

        if reverse { interleaved_grain.reverse() }
        interleaved_grain

    }

    pub fn generate_synthetic_grain(&self, osc_table: &[f32], freq: f32, sr: i32) -> Vec<f32> {

        let mut grain = vec![0.0; self.grain_size as usize];
        let sample_increment = freq * self.grain_size as f32 / sr as f32;
        
        let mut increment = 0.0;

        for (i, sample) in grain.iter_mut().enumerate() {

            if increment >= osc_table.len() as f32 - 1.0 { increment -= osc_table.len() as f32 }

            let intndx: f32 = increment.floor();
            let fracndx = increment - intndx;

            *sample = ((1.0 - fracndx) * osc_table[intndx as usize] + fracndx * osc_table[intndx as usize + 1]) * self.env[i] * self.amp;

            increment += sample_increment;

        }

        let mut interleaved_grain = vec![0.0; self.grain_size as usize * 2];
        for i in 0..self.grain_size as usize {
            interleaved_grain[i * 2] = grain[i] * self.left_pan_value;
            interleaved_grain[i * 2 + 1] = grain[i] * self.right_pan_value;
        }

        grain

    }

    pub fn build_grain_from_frame(&self, frame: &[f32], speed: f32, reverse: bool) -> Vec<f32> {
        self.generate_sound_grain(frame, 0.0, speed, reverse)

    }

}