use std::{
    fmt::{self, Display},
    sync::mpsc,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream,
};
use serde_repr::*;

use super::decode_zvei;

pub struct ZveiDecoder {
    stream: Stream,
    pub receiver: mpsc::Receiver<Tonfolge>,
}

unsafe impl Send for ZveiDecoder {}

impl ZveiDecoder {
    pub fn new<T>(
        audio_source: T,
        sample_rate: usize,
        size: usize,
        power_threshold: f32,
        clarity_threshold: f32,
    ) -> Result<Self, anyhow::Error>
    where
        T: AsRef<str>,
    {
        let host = cpal::default_host();
        let device = if audio_source.as_ref() == "default" {
            host.default_input_device().unwrap()
        } else {
            unimplemented!()
        };
        let config = device.default_input_config()?.into();
        let (sender, receiver) = mpsc::channel::<Tonfolge>();
        let input_fn = decode_zvei::decode_zvei(
            size,
            sample_rate,
            power_threshold,
            clarity_threshold,
            sender,
        );
        let err_fn = |e| eprintln!("{}", e);

        let stream = device.build_input_stream(&config, input_fn, err_fn)?;

        Ok(Self { stream, receiver })
    }
    pub fn start(&self) -> Result<(), anyhow::Error> {
        Ok(self.stream.play()?)
    }
}

pub type Tonfolge = Vec<Zvei>;

#[derive(PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr, Debug)]
#[repr(i8)]
pub enum Zvei {
    ONE = 1,
    TWO = 2,
    THREE = 3,
    FOUR = 4,
    FIVE = 5,
    SIX = 6,
    SEVEN = 7,
    EIGHT = 8,
    NINE = 9,
    ZERO = 0,
    RESERVE = -1,
}

impl Display for Zvei {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Zvei::ONE => write!(f, "1"),
            Zvei::TWO => write!(f, "2"),
            Zvei::THREE => write!(f, "3"),
            Zvei::FOUR => write!(f, "4"),
            Zvei::FIVE => write!(f, "5"),
            Zvei::SIX => write!(f, "6"),
            Zvei::SEVEN => write!(f, "7"),
            Zvei::EIGHT => write!(f, "8"),
            Zvei::NINE => write!(f, "9"),
            Zvei::ZERO => write!(f, "0"),
            Zvei::RESERVE => write!(f, "R"),
        }
    }
}

impl Zvei {
    pub fn new(frequency: f32) -> Option<Self> {
        match frequency as i32 {
            1055..=1065 => Some(Self::ONE),
            1150..=1170 => Some(Self::TWO),
            1260..=1280 => Some(Self::THREE),
            1390..=1410 => Some(Self::FOUR),
            1520..=1540 => Some(Self::FIVE),
            1660..=1680 => Some(Self::SIX),
            1820..=1840 => Some(Self::SEVEN),
            1990..=2010 => Some(Self::EIGHT),
            2190..=2210 => Some(Self::NINE),
            2390..=2410 => Some(Self::ZERO),
            2590..=2640 => Some(Self::RESERVE),
            _ => None,
        }
    }
}
