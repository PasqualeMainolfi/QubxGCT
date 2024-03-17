#![allow(dead_code, unused_imports)]

mod gct;

use gct::{ Envelope, EnvelopeMode, Grain, GrainParams, GrainSettings, ParamRange, ReverseMode, Table, TableMode };
use std::borrow::Borrow;
use std::{ fs::File, time::Duration };
use rand::Rng;
use qubx::{ Qubx, StreamParameters };
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{ Arc, Mutex };

enum GranulatorMode {
    Sound,
    Synthetic,
    Microphone
}

fn open_file(path: &str) -> Vec<f32> {
    let file = File::open(path).unwrap_or_else(|err| {
        println!("[ERROR] File {path} not found!, {err}");
        std::process::exit(1)
    });

    let (_, samples) = wav_io::read_from_file(file).unwrap();
    samples
}

const CHUNK: u32 = 4096;
const SR: i32 = 44100;

const FILES: [&str; 2] = [
    "./../audio_files_for_test/vox.wav", 
    "./../audio_files_for_test/suzanne_mono.wav"
    ];
    
const MODE: GranulatorMode = GranulatorMode::Microphone;

fn main() {

    let mut qubx = Qubx::new(true);
    qubx.start_monitoring_active_processes();

    let qparams: StreamParameters = StreamParameters {
        chunk: CHUNK,
        sr: SR,
        outchannels: 2,
        ..Default::default()
    };

    let mut grain_settings = GrainSettings {

        dur_range: ParamRange{ min: 0.01, max: 0.7 },
        freq_range: ParamRange{ min: 0.1, max: 2.5 },
        reverse: ReverseMode::RandomReverse,
        pan_range: ParamRange{ min: 0.1, max: 1.0 },
        amp_range: ParamRange{ min: 0.1, max: 0.7 },
        time_pos_range: ParamRange{ min: 0.0, max: 1.0},
        delay_range: ParamRange{ min: 0.01, max: 0.1 },
        sr: 44100,
        env_mode: Some(EnvelopeMode::Hanning)

    };

    let duration = 10 * SR;

    match MODE {
        
        GranulatorMode::Sound => {

            let master_out = qubx.create_master_streamout(String::from("MASTER"), qparams.clone());
            master_out.start(|_frame: &mut [f32]| {});
            let dsp_process = qubx.create_parallel_dsp_process(String::from("MASTER"));

            let audio1 = open_file(FILES[0]);
            let audio2 = open_file(FILES[1]);
            let audio_sigs: [&Vec<f32>; 2] = [&audio1, &audio2];
        
            let mut grain_params = GrainParams::new(Arc::new(grain_settings));

            let mut count = 0;
            loop {
                
                let grain = Grain::new(grain_params.dur, grain_params.amp, grain_params.pan_pos, grain_params.env_mode.clone());
                let g = grain.generate_sound_grain(audio_sigs[0], grain_params.time_pos, grain_params.freq, grain_params.rev);

                dsp_process.start(g, |_frame: &mut [f32]| {});

                let delay = grain_params.delay;
                std::thread::sleep(Duration::from_secs_f32(delay));
                grain_params.update_params();

                if count >= duration / CHUNK as i32 { 
                    qubx.close_qubx();
                    break;
                }
                count += 1;

            }

        }

        GranulatorMode::Synthetic => {

            let master_out = qubx.create_master_streamout(String::from("MASTER"), qparams.clone());
            master_out.start(|_frame: &mut [f32]| {});
            let dsp_process = qubx.create_parallel_dsp_process(String::from("MASTER"));
            
            grain_settings.freq_range = ParamRange { min: 70.0, max: 3000.0 };
            grain_settings.env_mode = Some(EnvelopeMode::Percussive);
            let mut grain_params = GrainParams::new(Arc::new(grain_settings));

            let wave_table = Table::table(TableMode::Sine, 4096);

            let mut count = 0;
            loop {
                
                let grain = Grain::new(grain_params.dur, grain_params.amp, grain_params.pan_pos, grain_params.env_mode.clone());
                let g = grain.generate_synthetic_grain(&wave_table, grain_params.freq, SR);

                dsp_process.start(g, |_frame: &mut [f32]| {});

                let delay = grain_params.delay;
                std::thread::sleep(Duration::from_secs_f32(delay));
                grain_params.update_params();

                if count >= duration / CHUNK as i32 { 
                    qubx.close_qubx();
                    break;
                }
                count += 1;

            }

        },

        GranulatorMode::Microphone => {

            let dsp_duplex = qubx.create_duplex_dsp_process(qparams.clone());
            
            let grain_settings = GrainSettings {
            
                dur_range: ParamRange{ min: 0.01, max: 3.0 },
                freq_range: ParamRange{ min: 0.5, max: 5.0 },
                reverse: ReverseMode::RandomReverse,
                pan_range: ParamRange{ min: 0.1, max: 1.0 },
                amp_range: ParamRange{ min: 0.5, max: 0.7 },
                time_pos_range: ParamRange{ min: 0.0, max: 0.0},
                delay_range: ParamRange{ min: 0.01, max: 0.3 },
                sr: 44100,
                env_mode: None
                
            };

            let grain_params = GrainParams::new(Arc::new(grain_settings));
            let grain_params_ptr = Arc::new(Mutex::new(grain_params));

            let dsp_function = {
                
                let grain_params_clone = Arc::clone(&grain_params_ptr);
                let gparams = grain_params_clone.lock().unwrap();
                
                let mut grain_length: usize = (((gparams.dur as f32 / CHUNK as f32).ceil()) * CHUNK as f32) as usize;
                let mut env = Envelope::generate_envelope(EnvelopeMode::Hanning, grain_length);
                let mut tot_delay_frame_silence = (gparams.delay * SR as f32 / CHUNK as f32).ceil() as i32;
                
                let mut hop = 0;
                let mut delay_frame_silence_count = 0;
                
                move |frame: &[f32]| {
                    
                    let gparams_for_closure = Arc::clone(&grain_params_ptr);
                    let mut gparams = gparams_for_closure.lock().unwrap();
                    
                    match hop.cmp(&grain_length) {

                        std::cmp::Ordering::Less => {
                            let grain = Grain::new(frame.len() as i32, gparams.amp, gparams.pan_pos, None);
                            let mut g = grain.generate_sound_grain(frame, gparams.time_pos, gparams.freq, gparams.rev);
                            let start = hop;

                            for i in 0..frame.len() {
                                g[i * 2] *= env[start + i];
                                g[i * 2 + 1] *= env[start + i];
                            }
                            
                            hop += CHUNK as usize;
                            g
                        },

                        std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => {
                            if delay_frame_silence_count < tot_delay_frame_silence {
                                delay_frame_silence_count += 1
                            } else {
                                hop = 0;
                                delay_frame_silence_count = 0;
                                gparams.update_params();
                                grain_length = (((gparams.dur as f32 / CHUNK as f32).ceil()) * CHUNK as f32) as usize;
                                env = Envelope::generate_envelope(EnvelopeMode::Hanning, grain_length);
                                tot_delay_frame_silence = (gparams.delay * SR as f32 / CHUNK as f32).ceil() as i32;
                            };
                            let mut ysilence = vec![0.0; frame.len() * 2];
                            for (i, sample) in frame.iter().enumerate() {
                                ysilence[i * 2] = *sample * 0.0;
                                ysilence[i * 2 + 1] = *sample * 0.0 ;
                            }
                            ysilence
                        }
                        
                    }
                    
                }
            };
            
            // in (mono) -> out (stereo)
            
            dsp_duplex.start(dsp_function);
            
            let mut count = 0;
            loop {

                if count >= duration { 
                    qubx.close_qubx();
                    break;
                }
                count += 1;
                std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / 2.0));
            
            }

        }
    }





}
