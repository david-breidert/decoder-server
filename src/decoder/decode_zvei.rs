use std::{sync::mpsc, time::Instant};

use cpal::InputCallbackInfo;
use pitch_detection::detector::{mcleod::McLeodDetector, PitchDetector};

use super::{Tonfolge, Zvei};

pub fn decode_zvei(
    size: usize,
    sample_rate: usize,
    power_threshold: f32,
    clarity_threshold: f32,
    sender: mpsc::Sender<Tonfolge>,
) -> impl FnMut(&[f32], &InputCallbackInfo) {
    let padding = size / 2;
    let mut signal = vec![0.0; 0];
    let mut detector: McLeodDetector<f32> = McLeodDetector::new(size, padding);
    let mut tf = Vec::<Zvei>::new();
    let mut last_sound = Instant::now();

    move |data: &[f32], _: &cpal::InputCallbackInfo| {
        signal.extend_from_slice(data);

        if signal.len() >= size {
            if let Some(pitch) = detector.get_pitch(
                &signal[..size],
                sample_rate,
                power_threshold,
                clarity_threshold,
            ) {
                // for some reason this returns half of the expected frequency
                if let Some(zvei) = Zvei::new(pitch.frequency * 2.0) {
                    last_sound = Instant::now();
                    tf.push(zvei);
                }
            }
            signal.clear();

            if last_sound.elapsed().as_millis() > 500 && !tf.is_empty() {
                let mut count: u8 = 0;
                let mut res_tf = Vec::with_capacity(5);
                for (i, z) in tf.iter().enumerate() {
                    count += 1;
                    if i == tf.len() - 1 || *z != tf[i + 1] {
                        if count >= 2 {
                            res_tf.push(*z);
                        }
                        count = 0;
                    }
                }
                if res_tf.len() == 5 {
                    // print!("{} - ", time.format("%H:%M:%S"));
                    let mut current_zvei = &Zvei::RESERVE;
                    for v in &mut res_tf {
                        if *v == Zvei::RESERVE {
                            *v = *current_zvei;
                        }
                        current_zvei = v;
                    }
                    sender.send(res_tf).unwrap_or_else(|e| eprintln!("{}", e));
                }

                tf.clear();
            }
        }
    }
}
