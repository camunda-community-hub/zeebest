use crate::gateway;
use crate::BrokerRole;

/// Describes a partition on a broker.
#[derive(Debug)]
pub struct Partition {
    pub partition_id: i32,
    pub role: BrokerRole,
}

impl From<gateway::Partition> for Partition {
    fn from(p: gateway::Partition) -> Self {
        let role = match p.role {
            0 => BrokerRole::LEADER,
            1 => BrokerRole::FOLLOWER,
            _ => unreachable!(),
        };
        Self {
            partition_id: p.partition_id,
            role,
        }
    }
}
