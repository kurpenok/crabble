use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use tokio::sync::{mpsc, Mutex};

use crate::error::BrokerError;
use crate::utils::log_info;

pub struct Broker {
    channels: Arc<Mutex<HashMap<String, Vec<(u64, mpsc::Sender<Bytes>)>>>>,
    sub_id_counter: AtomicU64,
}

impl Broker {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
            sub_id_counter: AtomicU64::new(1),
        }
    }

    pub async fn create_channel(&self, name: String) -> Result<(), BrokerError> {
        let mut channels = self.channels.lock().await;
        if channels.contains_key(&name) {
            return Err(BrokerError::ChannelAlreadyExists(name));
        }
        channels.insert(name.clone(), Vec::new());
        log_info(&format!("Channel '{}' created", name));
        Ok(())
    }

    pub async fn subscribe(
        &self,
        channel: &str,
    ) -> Result<(u64, mpsc::Receiver<Bytes>), BrokerError> {
        let mut channels = self.channels.lock().await;
        match channels.get_mut(channel) {
            Some(subscribers) => {
                let (tx, rx) = mpsc::channel(100);
                let id = self.sub_id_counter.fetch_add(1, Ordering::Relaxed);
                subscribers.push((id, tx));
                log_info(&format!("Subscriber {} added to channel '{}'", id, channel));
                Ok((id, rx))
            }
            None => Err(BrokerError::ChannelNotFound(channel.to_string())),
        }
    }

    pub async fn unsubscribe(&self, channel: &str, sub_id: u64) -> Result<(), BrokerError> {
        let mut channels = self.channels.lock().await;
        if let Some(subscribers) = channels.get_mut(channel) {
            let orig_len = subscribers.len();
            subscribers.retain(|(id, _)| *id != sub_id);
            if subscribers.len() == orig_len {
                return Err(BrokerError::SubscriberNotFound(sub_id, channel.to_string()));
            }
            log_info(&format!(
                "Subscriber {} removed from channel '{}'",
                sub_id, channel
            ));
            Ok(())
        } else {
            Err(BrokerError::ChannelNotFound(channel.to_string()))
        }
    }

    pub async fn publish(&self, channel: &str, message: Bytes) -> Result<(), BrokerError> {
        let channels = self.channels.lock().await;
        match channels.get(channel) {
            Some(subscribers) => {
                log_info(&format!("Publishing message to channel '{}'...", channel));
                for &(_, ref sub) in subscribers {
                    if let Err(e) = sub.send(message.clone()).await {
                        log_info(&format!("Error sending message: {}", e));
                    }
                }
                Ok(())
            }
            None => Err(BrokerError::ChannelNotFound(channel.to_string())),
        }
    }
}
