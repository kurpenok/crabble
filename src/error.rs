use std::fmt;

#[derive(Debug)]
pub enum BrokerError {
    ChannelAlreadyExists(String),
    ChannelNotFound(String),
    SubscriberNotFound(u64, String),
}

impl fmt::Display for BrokerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrokerError::ChannelAlreadyExists(name) => {
                write!(f, "Channel '{}' already exists", name)
            }
            BrokerError::ChannelNotFound(name) => write!(f, "Channel '{}' not found", name),
            BrokerError::SubscriberNotFound(id, name) => {
                write!(f, "Subscriber '{}' not found in channel '{}'", id, name)
            }
        }
    }
}

impl std::error::Error for BrokerError {}
