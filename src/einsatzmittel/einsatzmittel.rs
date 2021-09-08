use std::path::Path;

use crate::decoder::Tonfolge;

use serde::{Deserialize, Serialize};

use tokio::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct Einsatzmittel {
    pub einsatzmittel: String,
    pub tonfolge: Tonfolge,
}

impl Einsatzmittel {
    pub async fn init() -> Vec<Self> {
        // let em = Vec::new();
        let path = Path::new("einsatzmittel.json");
        let file = fs::read(path).await.unwrap();
        serde_json::from_slice(&file).unwrap()
    }
}
