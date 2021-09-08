mod decoder;
mod einsatzmittel;
mod server;

use chrono::Local;
use decoder::ZveiDecoder;
use einsatzmittel::Einsatzmittel;
use server::Server;

const SAMPLE_RATE: usize = 48000;
const SIZE: usize = 1024;
const POWER_THRESHOLD: f32 = 3.0;
const CLARITY_THRESHOLD: f32 = 0.5;

#[tokio::main]
async fn main() -> ! {
    let server = Server::new();
    let sender = server.sender.clone();
    tokio::spawn(server.run());
    let em = Einsatzmittel::init().await;

    let zvei_decoder = ZveiDecoder::new(
        "default",
        SAMPLE_RATE,
        SIZE,
        POWER_THRESHOLD,
        CLARITY_THRESHOLD,
    )
    .unwrap();

    zvei_decoder.start().unwrap();

    loop {
        let s = zvei_decoder.receiver.recv().unwrap();
        let time = Local::now();
        let mut msg = String::new();

        msg.push_str(&format!("{} - ", time.format("%H:%M:%S")));

        let mut found_em = false;
        for e in &em {
            if e.tonfolge == s {
                msg.push_str(&e.einsatzmittel);
                found_em = true;
            }
        }

        if !found_em {
            for z in s {
                msg.push_str(&format!("{}", z));
            }
        }
        println!("{}", msg);
        if sender.receiver_count() > 0 {
            sender.send(msg).expect("Could not send sender");
        }
    }
}
