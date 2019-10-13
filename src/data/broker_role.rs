pub use crate::gateway;

/// Is this broker a leader or not?
#[derive(Debug)]
pub enum BrokerRole {
    LEADER = 0,
    FOLLOWER = 1,
}

impl From<gateway::partition::PartitionBrokerRole> for BrokerRole {
    fn from(pbr: gateway::partition::PartitionBrokerRole) -> Self {
        match pbr {
            gateway::partition::PartitionBrokerRole::Follower => BrokerRole::FOLLOWER,
            gateway::partition::PartitionBrokerRole::Leader => BrokerRole::LEADER,
        }
    }
}
