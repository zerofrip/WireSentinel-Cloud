use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CloudEvent {
    NodeFailed(NodeFailed),
    FailoverTriggered(FailoverTriggered),
    BillingSecurityViolation(BillingSecurityViolation),
    ZtnaSecurityViolation(ZtnaSecurityViolation),
    IdentitySecurityViolation(IdentitySecurityViolation),
    SseSecurityViolation(SseSecurityViolation),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeFailed {
    pub node_id: String,
    pub node_name: String,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailoverTriggered {
    pub previous_leader_id: Option<String>,
    pub new_leader_id: String,
    pub lease_key: String,
    pub triggered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BillingSecurityViolation {
    pub tenant_id: Option<String>,
    pub violation: String,
    pub source: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZtnaSecurityViolation {
    pub tenant_id: String,
    pub subject_id: Option<String>,
    pub resource_id: Option<String>,
    pub decision: String,
    pub reason: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentitySecurityViolation {
    pub tenant_id: String,
    pub provider_id: Option<String>,
    pub subject: Option<String>,
    pub violation: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SseSecurityViolation {
    pub tenant_id: String,
    pub device_id: Option<String>,
    pub violation_kind: String,
    pub resource: Option<String>,
    pub action: String,
    pub reason: String,
    pub detected_at: DateTime<Utc>,
}

impl CloudEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::NodeFailed(_) => "node_failed",
            Self::FailoverTriggered(_) => "failover_triggered",
            Self::BillingSecurityViolation(_) => "billing_security_violation",
            Self::ZtnaSecurityViolation(_) => "ztna_security_violation",
            Self::IdentitySecurityViolation(_) => "identity_security_violation",
            Self::SseSecurityViolation(_) => "sse_security_violation",
        }
    }
}
