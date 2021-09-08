use std::{
    convert::Infallible,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::SinkExt;
use tokio::{
    sync::{broadcast, mpsc, Mutex},
    time,
};
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
                        println!("New client! ID: {}", my_id);

                        let socket = Arc::new(Mutex::new(socket));
                        let socket_clone = socket.clone();

                        let (break_tx, mut break_rx) = mpsc::channel::<bool>(1);

                        tokio::spawn(async move {
                            loop {
                                time::sleep(Duration::from_secs(5)).await;
                                if let Err(_) = socket_clone.lock().await
                                    .send(Message::ping("--heartbeat--"))
                                    .await
                                {
                                    break_tx.send(true).await.unwrap();
                                    break;
                                };
                            }
                        });

                        loop {
                            tokio::select! {
                                s = receiver.recv() => {
                                    let string = s.unwrap();
                                    if let Err(_) = socket.lock().await.send(Message::text(string)).await {
                                        println!("Client: {} disconnected", my_id);
                                        break;
                                    }
                                }
                                _ = break_rx.recv() => {
                                    println!("Client {} disconnected", my_id);
                                    break;
                                }
                            }
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
