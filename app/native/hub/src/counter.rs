use crate::messages;
use crate::tokio;
use messages::counter::*;
use rinf::debug_print;

pub async fn set_counter() {
    let mut receiver = SetCounter::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let set_counter = dart_signal.message;

        let mut counter = models::Counter::new();
        counter.set(set_counter.counter);

        if let Err(e) = api_client::set_counter(&counter).await {
            debug_print!("api_client::set_counter() error: {e}");
        }
    }
}

pub async fn counter() {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        match api_client::get_counter().await {
            Ok(counter) => {
                Counter {
                    number: counter.get(),
                }
                .send_signal_to_dart(None);
            }
            Err(e) => {
                debug_print!("api_client::get_counter() error: {e}");
            }
        }
    }
}
