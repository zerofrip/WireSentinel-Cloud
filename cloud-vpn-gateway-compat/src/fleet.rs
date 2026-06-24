use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiresockRollupPayload {
    pub reporting_endpoints: i64,
    pub active_split_templates: i64,
    pub tcp_termination_rules: i64,
    pub handshake_proxy_active: i64,
    pub bypass_events: i64,
    pub fleet_health_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiresockFleetRollup {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub reporting_endpoints: i64,
    pub active_split_templates: i64,
    pub tcp_termination_rules: i64,
    pub handshake_proxy_active: i64,
    pub bypass_events: i64,
    pub fleet_health_score: f64,
    pub rollup: serde_json::Value,
    pub rolled_up_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiresockFleetOverview {
    pub tenant_id: String,
    pub reporting_endpoints: i64,
    pub active_split_templates: i64,
    pub tcp_termination_rules: i64,
    pub handshake_proxy_active: i64,
    pub bypass_events: i64,
    pub fleet_health_score: f64,
    pub controllers_reporting: i64,
    pub rollups: Vec<WiresockFleetRollup>,
    pub split_templates: Vec<WiresockSplitTemplateRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiresockSplitTemplateRecord {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub name: String,
    pub description: String,
    pub template_mode: String,
    pub enabled: bool,
    pub app_rules_count: i64,
    pub domain_rules_count: i64,
    pub content: serde_json::Value,
    pub synced_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiresockTcpTerminationRecord {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub mode: String,
    pub rule_name: String,
    pub process_name: Option<String>,
    pub profile_id: Option<String>,
    pub enabled: bool,
    pub content: serde_json::Value,
    pub synced_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiresockHandshakeProxyRecord {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub name: String,
    pub proxy_type: String,
    pub endpoint: Option<String>,
    pub enabled: bool,
    pub content: serde_json::Value,
    pub synced_at: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct WiresockFleetMonitor {
    pool: DbPool,
}

impl WiresockFleetMonitor {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn record_rollup(
        &self,
        tenant_id: &str,
        controller_id: Option<&str>,
        payload: &WiresockRollupPayload,
    ) -> Result<WiresockFleetRollup, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let rollup_json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO tenant_wiresock_analytics_rollups (
                id, tenant_id, controller_id, reporting_endpoints, active_split_templates,
                tcp_termination_rules, handshake_proxy_active, bypass_events, fleet_health_score,
                rollup_json, rolled_up_at, created_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(controller_id)
        .bind(payload.reporting_endpoints)
        .bind(payload.active_split_templates)
        .bind(payload.tcp_termination_rules)
        .bind(payload.handshake_proxy_active)
        .bind(payload.bypass_events)
        .bind(payload.fleet_health_score)
        .bind(&rollup_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(WiresockFleetRollup {
            id,
            tenant_id: tenant_id.to_string(),
            controller_id: controller_id.map(str::to_string),
            reporting_endpoints: payload.reporting_endpoints,
            active_split_templates: payload.active_split_templates,
            tcp_termination_rules: payload.tcp_termination_rules,
            handshake_proxy_active: payload.handshake_proxy_active,
            bypass_events: payload.bypass_events,
            fleet_health_score: payload.fleet_health_score,
            rollup: serde_json::from_str(&rollup_json).unwrap_or(serde_json::json!({})),
            rolled_up_at: now.clone(),
            created_at: now,
        })
    }

    pub async fn fleet_overview(&self, tenant_id: &str) -> Result<WiresockFleetOverview, DbError> {
        let rollups = self.list_rollups(tenant_id, Some(50)).await?;
        let split_templates = self.list_split_templates(tenant_id, Some(50)).await?;
        let controllers_reporting = rollups
            .iter()
            .filter_map(|r| r.controller_id.as_deref())
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        let fleet_health_score = if rollups.is_empty() {
            0.0
        } else {
            rollups
                .iter()
                .map(|r| r.fleet_health_score)
                .sum::<f64>()
                / rollups.len() as f64
        };

        Ok(WiresockFleetOverview {
            tenant_id: tenant_id.to_string(),
            reporting_endpoints: rollups.iter().map(|r| r.reporting_endpoints).sum(),
            active_split_templates: rollups.iter().map(|r| r.active_split_templates).sum(),
            tcp_termination_rules: rollups.iter().map(|r| r.tcp_termination_rules).sum(),
            handshake_proxy_active: rollups.iter().map(|r| r.handshake_proxy_active).sum(),
            bypass_events: rollups.iter().map(|r| r.bypass_events).sum(),
            fleet_health_score,
            controllers_reporting,
            rollups,
            split_templates,
        })
    }

    pub async fn list_split_templates(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<WiresockSplitTemplateRecord>, DbError> {
        let limit = limit.unwrap_or(100);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            String,
            i64,
            i64,
            i64,
            String,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, name, description, template_mode, enabled,
                    app_rules_count, domain_rules_count, content_json, synced_at, created_at,
                    updated_at
             FROM tenant_wiresock_split_templates WHERE tenant_id = ? ORDER BY synced_at DESC LIMIT ?",
        )
        .bind(tenant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    tenant_id,
                    controller_id,
                    name,
                    description,
                    template_mode,
                    enabled,
                    app_rules_count,
                    domain_rules_count,
                    content_json,
                    synced_at,
                    created_at,
                    updated_at,
                )| {
                    WiresockSplitTemplateRecord {
                        id,
                        tenant_id,
                        controller_id,
                        name,
                        description,
                        template_mode,
                        enabled: enabled != 0,
                        app_rules_count,
                        domain_rules_count,
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        synced_at,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn list_tcp_termination(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<WiresockTcpTerminationRecord>, DbError> {
        let limit = limit.unwrap_or(100);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            Option<String>,
            i64,
            String,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, mode, rule_name, process_name, profile_id,
                    enabled, content_json, synced_at, created_at, updated_at
             FROM tenant_wiresock_tcp_termination WHERE tenant_id = ? ORDER BY synced_at DESC LIMIT ?",
        )
        .bind(tenant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    tenant_id,
                    controller_id,
                    mode,
                    rule_name,
                    process_name,
                    profile_id,
                    enabled,
                    content_json,
                    synced_at,
                    created_at,
                    updated_at,
                )| {
                    WiresockTcpTerminationRecord {
                        id,
                        tenant_id,
                        controller_id,
                        mode,
                        rule_name,
                        process_name,
                        profile_id,
                        enabled: enabled != 0,
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        synced_at,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn list_handshake_proxy(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<WiresockHandshakeProxyRecord>, DbError> {
        let limit = limit.unwrap_or(100);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            i64,
            String,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, name, proxy_type, endpoint, enabled,
                    content_json, synced_at, created_at, updated_at
             FROM tenant_wiresock_handshake_proxy WHERE tenant_id = ? ORDER BY synced_at DESC LIMIT ?",
        )
        .bind(tenant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    tenant_id,
                    controller_id,
                    name,
                    proxy_type,
                    endpoint,
                    enabled,
                    content_json,
                    synced_at,
                    created_at,
                    updated_at,
                )| {
                    WiresockHandshakeProxyRecord {
                        id,
                        tenant_id,
                        controller_id,
                        name,
                        proxy_type,
                        endpoint,
                        enabled: enabled != 0,
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        synced_at,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    async fn list_rollups(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<WiresockFleetRollup>, DbError> {
        let limit = limit.unwrap_or(50);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            i64,
            i64,
            i64,
            i64,
            i64,
            f64,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, reporting_endpoints, active_split_templates,
                    tcp_termination_rules, handshake_proxy_active, bypass_events,
                    fleet_health_score, rollup_json, rolled_up_at, created_at
             FROM tenant_wiresock_analytics_rollups WHERE tenant_id = ? ORDER BY rolled_up_at DESC LIMIT ?",
        )
        .bind(tenant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    tenant_id,
                    controller_id,
                    reporting_endpoints,
                    active_split_templates,
                    tcp_termination_rules,
                    handshake_proxy_active,
                    bypass_events,
                    fleet_health_score,
                    rollup_json,
                    rolled_up_at,
                    created_at,
                )| {
                    WiresockFleetRollup {
                        id,
                        tenant_id,
                        controller_id,
                        reporting_endpoints,
                        active_split_templates,
                        tcp_termination_rules,
                        handshake_proxy_active,
                        bypass_events,
                        fleet_health_score,
                        rollup: serde_json::from_str(&rollup_json)
                            .unwrap_or(serde_json::json!({})),
                        rolled_up_at,
                        created_at,
                    }
                },
            )
            .collect())
    }
}
