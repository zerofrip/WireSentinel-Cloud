use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRollupPayload {
    pub reporting_agents: i64,
    pub open_investigations: i64,
    pub critical_risks: i64,
    pub total_correlations: i64,
    pub compliance_pct: f64,
    pub avg_risk_score: f64,
    pub prompt_injection_events: i64,
    pub data_exfiltration_events: i64,
    pub fleet_ai_risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiFleetRollup {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub reporting_agents: i64,
    pub open_investigations: i64,
    pub critical_risks: i64,
    pub total_correlations: i64,
    pub compliance_pct: f64,
    pub avg_risk_score: f64,
    pub prompt_injection_events: i64,
    pub data_exfiltration_events: i64,
    pub fleet_ai_risk_score: f64,
    pub rollup: serde_json::Value,
    pub rolled_up_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiFleetOverview {
    pub tenant_id: String,
    pub reporting_agents: i64,
    pub open_investigations: i64,
    pub critical_risks: i64,
    pub total_correlations: i64,
    pub compliance_pct: f64,
    pub avg_risk_score: f64,
    pub prompt_injection_events: i64,
    pub data_exfiltration_events: i64,
    pub fleet_ai_risk_score: f64,
    pub controllers_reporting: i64,
    pub rollups: Vec<AiFleetRollup>,
    pub investigations: Vec<AiInvestigationRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiInvestigationRecord {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub title: String,
    pub status: String,
    pub severity: String,
    pub category: String,
    pub model_name: Option<String>,
    pub agent_id: Option<String>,
    pub finding_count: i64,
    pub content: serde_json::Value,
    pub opened_at: String,
    pub resolved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRiskRecord {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub risk_category: String,
    pub risk_score: f64,
    pub severity: String,
    pub model_name: Option<String>,
    pub resource_id: Option<String>,
    pub status: String,
    pub content: serde_json::Value,
    pub assessed_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiReportRecord {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub report_type: String,
    pub title: String,
    pub status: String,
    pub compliance_pct: f64,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub content: serde_json::Value,
    pub generated_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCorrelationRecord {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub correlation_key: String,
    pub event_count: i64,
    pub severity: String,
    pub status: String,
    pub source_kinds: serde_json::Value,
    pub content: serde_json::Value,
    pub correlated_at: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct AiFleetMonitor {
    pool: DbPool,
}

impl AiFleetMonitor {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn record_rollup(
        &self,
        tenant_id: &str,
        controller_id: Option<&str>,
        payload: &AiRollupPayload,
    ) -> Result<AiFleetRollup, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let rollup_json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO tenant_ai_analytics_rollups (
                id, tenant_id, controller_id, reporting_agents, open_investigations,
                critical_risks, total_correlations, compliance_pct, avg_risk_score,
                prompt_injection_events, data_exfiltration_events, fleet_ai_risk_score,
                rollup_json, rolled_up_at, created_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(controller_id)
        .bind(payload.reporting_agents)
        .bind(payload.open_investigations)
        .bind(payload.critical_risks)
        .bind(payload.total_correlations)
        .bind(payload.compliance_pct)
        .bind(payload.avg_risk_score)
        .bind(payload.prompt_injection_events)
        .bind(payload.data_exfiltration_events)
        .bind(payload.fleet_ai_risk_score)
        .bind(&rollup_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(AiFleetRollup {
            id,
            tenant_id: tenant_id.to_string(),
            controller_id: controller_id.map(str::to_string),
            reporting_agents: payload.reporting_agents,
            open_investigations: payload.open_investigations,
            critical_risks: payload.critical_risks,
            total_correlations: payload.total_correlations,
            compliance_pct: payload.compliance_pct,
            avg_risk_score: payload.avg_risk_score,
            prompt_injection_events: payload.prompt_injection_events,
            data_exfiltration_events: payload.data_exfiltration_events,
            fleet_ai_risk_score: payload.fleet_ai_risk_score,
            rollup: serde_json::from_str(&rollup_json).unwrap_or(serde_json::json!({})),
            rolled_up_at: now.clone(),
            created_at: now,
        })
    }

    pub async fn fleet_overview(&self, tenant_id: &str) -> Result<AiFleetOverview, DbError> {
        let rollups = self.list_rollups(tenant_id, Some(50)).await?;
        let investigations = self.list_investigations(tenant_id, Some(50)).await?;
        let controllers_reporting = rollups
            .iter()
            .filter_map(|r| r.controller_id.as_deref())
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        let compliance_pct = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.compliance_pct).sum::<f64>() / rollups.len() as f64
        };

        let avg_risk_score = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.avg_risk_score).sum::<f64>() / rollups.len() as f64
        };

        let fleet_ai_risk_score = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.fleet_ai_risk_score).sum::<f64>() / rollups.len() as f64
        };

        Ok(AiFleetOverview {
            tenant_id: tenant_id.to_string(),
            reporting_agents: rollups.iter().map(|r| r.reporting_agents).sum(),
            open_investigations: rollups.iter().map(|r| r.open_investigations).sum(),
            critical_risks: rollups.iter().map(|r| r.critical_risks).sum(),
            total_correlations: rollups.iter().map(|r| r.total_correlations).sum(),
            compliance_pct,
            avg_risk_score,
            prompt_injection_events: rollups.iter().map(|r| r.prompt_injection_events).sum(),
            data_exfiltration_events: rollups.iter().map(|r| r.data_exfiltration_events).sum(),
            fleet_ai_risk_score,
            controllers_reporting,
            rollups,
            investigations,
        })
    }

    pub async fn list_investigations(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<AiInvestigationRecord>, DbError> {
        let limit = limit.unwrap_or(100);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            i64,
            String,
            String,
            Option<String>,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, title, status, severity, category, model_name,
                    agent_id, finding_count, content_json, opened_at, resolved_at, created_at,
                    updated_at
             FROM tenant_ai_investigations WHERE tenant_id = ? ORDER BY opened_at DESC LIMIT ?",
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
                    title,
                    status,
                    severity,
                    category,
                    model_name,
                    agent_id,
                    finding_count,
                    content_json,
                    opened_at,
                    resolved_at,
                    created_at,
                    updated_at,
                )| {
                    AiInvestigationRecord {
                        id,
                        tenant_id,
                        controller_id,
                        title,
                        status,
                        severity,
                        category,
                        model_name,
                        agent_id,
                        finding_count,
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        opened_at,
                        resolved_at,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn list_risk(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<AiRiskRecord>, DbError> {
        let limit = limit.unwrap_or(100);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            f64,
            String,
            Option<String>,
            Option<String>,
            String,
            String,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, risk_category, risk_score, severity, model_name,
                    resource_id, status, content_json, assessed_at, created_at, updated_at
             FROM tenant_ai_risk WHERE tenant_id = ? ORDER BY assessed_at DESC LIMIT ?",
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
                    risk_category,
                    risk_score,
                    severity,
                    model_name,
                    resource_id,
                    status,
                    content_json,
                    assessed_at,
                    created_at,
                    updated_at,
                )| {
                    AiRiskRecord {
                        id,
                        tenant_id,
                        controller_id,
                        risk_category,
                        risk_score,
                        severity,
                        model_name,
                        resource_id,
                        status,
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        assessed_at,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn list_reports(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<AiReportRecord>, DbError> {
        let limit = limit.unwrap_or(100);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            String,
            f64,
            Option<String>,
            Option<String>,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, report_type, title, status, compliance_pct,
                    period_start, period_end, content_json, generated_at, created_at
             FROM tenant_ai_reports WHERE tenant_id = ? ORDER BY generated_at DESC LIMIT ?",
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
                    report_type,
                    title,
                    status,
                    compliance_pct,
                    period_start,
                    period_end,
                    content_json,
                    generated_at,
                    created_at,
                )| {
                    AiReportRecord {
                        id,
                        tenant_id,
                        controller_id,
                        report_type,
                        title,
                        status,
                        compliance_pct,
                        period_start,
                        period_end,
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        generated_at,
                        created_at,
                    }
                },
            )
            .collect())
    }

    pub async fn list_correlations(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<AiCorrelationRecord>, DbError> {
        let limit = limit.unwrap_or(100);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            i64,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, correlation_key, event_count, severity, status,
                    source_kinds_json, content_json, correlated_at, created_at, updated_at
             FROM tenant_ai_correlations WHERE tenant_id = ? ORDER BY correlated_at DESC LIMIT ?",
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
                    correlation_key,
                    event_count,
                    severity,
                    status,
                    source_kinds_json,
                    content_json,
                    correlated_at,
                    created_at,
                    updated_at,
                )| {
                    AiCorrelationRecord {
                        id,
                        tenant_id,
                        controller_id,
                        correlation_key,
                        event_count,
                        severity,
                        status,
                        source_kinds: serde_json::from_str(&source_kinds_json)
                            .unwrap_or(serde_json::json!([])),
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        correlated_at,
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
    ) -> Result<Vec<AiFleetRollup>, DbError> {
        let limit = limit.unwrap_or(50);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            i64,
            i64,
            i64,
            i64,
            f64,
            f64,
            i64,
            i64,
            f64,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, reporting_agents, open_investigations,
                    critical_risks, total_correlations, compliance_pct, avg_risk_score,
                    prompt_injection_events, data_exfiltration_events, fleet_ai_risk_score,
                    rollup_json, rolled_up_at, created_at
             FROM tenant_ai_analytics_rollups WHERE tenant_id = ? ORDER BY rolled_up_at DESC LIMIT ?",
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
                    reporting_agents,
                    open_investigations,
                    critical_risks,
                    total_correlations,
                    compliance_pct,
                    avg_risk_score,
                    prompt_injection_events,
                    data_exfiltration_events,
                    fleet_ai_risk_score,
                    rollup_json,
                    rolled_up_at,
                    created_at,
                )| {
                    AiFleetRollup {
                        id,
                        tenant_id,
                        controller_id,
                        reporting_agents,
                        open_investigations,
                        critical_risks,
                        total_correlations,
                        compliance_pct,
                        avg_risk_score,
                        prompt_injection_events,
                        data_exfiltration_events,
                        fleet_ai_risk_score,
                        rollup: serde_json::from_str(&rollup_json).unwrap_or(serde_json::json!({})),
                        rolled_up_at,
                        created_at,
                    }
                },
            )
            .collect())
    }
}
