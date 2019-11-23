use crate::gateway;
use crate::BrokerInfo;

/// The topology of the zeebe cluster.
#[derive(Debug)]
pub struct Topology {
    pub brokers: Vec<BrokerInfo>,
}

impl Topology {
    pub fn from_raw(topology_response: gateway::TopologyResponse) -> Topology {
        Self {
            brokers: topology_response
                .brokers
                .into_iter()
                .map(From::from)
                .collect(),
        }
    }
}

impl From<gateway::TopologyResponse> for Topology {
    fn from(tr: gateway::TopologyResponse) -> Self {
        Self {
            brokers: tr.brokers.into_iter().map(From::from).collect(),
        }
    }
}
