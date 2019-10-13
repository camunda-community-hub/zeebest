use crate::gateway;
use crate::Partition;

/// Describes a zeebe broker.
#[derive(Debug)]
pub struct BrokerInfo {
    pub node_id: i32,
    pub host: String,
    pub port: i32,
    pub partitions: Vec<Partition>,
}

impl From<gateway::BrokerInfo> for BrokerInfo {
    fn from(bi: gateway::BrokerInfo) -> Self {
        Self {
            node_id: bi.node_id,
            host: bi.host,
            port: bi.port,
            partitions: bi.partitions.into_iter().map(From::from).collect(),
        }
    }
}
