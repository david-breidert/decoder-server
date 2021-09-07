use std::{
    convert::Infallible,
    sync::atomic::{AtomicUsize, Ordering},
};

use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use warp::{ws::Message, Filter};

static NEXT_CONN_ID: AtomicUsize = AtomicUsize::new(1);

pub struct Server {
    port: u16,
    pub sender: broadcast::Sender<String>,
}

impl Server {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(5);
        Self { port: 8000, sender }
    }

    pub async fn run(self) {
        let routes = warp::path("alarm")
            .and(warp::ws())
            .and(with_receiver(self.sender.clone()))
            .map(
                |ws: warp::ws::Ws, mut receiver: broadcast::Receiver<String>| {
                    ws.on_upgrade(move |socket| async move {
                        let my_id = NEXT_CONN_ID.fetch_add(1, Ordering::Relaxed);
                        println!("New connection: {}", my_id);
                        let (mut ws_tx, _) = socket.split();

                        loop {
                            let s = receiver.recv().await.unwrap();
                            ws_tx
                                .send(Message::text(s))
                                .await
                                .expect("Could not send ws");
                        }
                    })
                },
            );

        warp::serve(routes).run(([0, 0, 0, 0], self.port)).await
    }
}

fn with_receiver(
    tx: broadcast::Sender<String>,
) -> impl Filter<Extract = (broadcast::Receiver<String>,), Error = Infallible> + Clone {
    warp::any().map(move || tx.subscribe())
}
