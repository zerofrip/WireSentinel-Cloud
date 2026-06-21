use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CnappRollupPayload {
    pub reporting_accounts: i64,
    pub posture_score: f64,
    pub compliance_pct: f64,
    pub open_vulnerabilities: i64,
    pub critical_vulnerabilities: i64,
    pub attack_paths_detected: i64,
    pub multi_cloud_providers: i64,
    pub fleet_risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CnappFleetRollup {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub reporting_accounts: i64,
    pub posture_score: f64,
    pub compliance_pct: f64,
    pub open_vulnerabilities: i64,
    pub critical_vulnerabilities: i64,
    pub attack_paths_detected: i64,
    pub multi_cloud_providers: i64,
    pub fleet_risk_score: f64,
    pub rollup: serde_json::Value,
    pub rolled_up_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CnappFleetOverview {
    pub tenant_id: String,
    pub reporting_accounts: i64,
    pub posture_score: f64,
    pub compliance_pct: f64,
    pub open_vulnerabilities: i64,
    pub critical_vulnerabilities: i64,
    pub attack_paths_detected: i64,
    pub multi_cloud_providers: i64,
    pub fleet_risk_score: f64,
    pub controllers_reporting: i64,
    pub rollups: Vec<CnappFleetRollup>,
    pub attack_paths: Vec<CnappAttackPathRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CnappPostureRecord {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub cloud_provider: String,
    pub account_id: Option<String>,
    pub resource_kind: String,
    pub posture_score: f64,
    pub risk_level: String,
    pub findings_count: i64,
    pub content: serde_json::Value,
    pub assessed_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CnappComplianceRecord {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub framework: String,
    pub control_id: String,
    pub control_name: String,
    pub status: String,
    pub compliance_pct: f64,
    pub last_checked_at: Option<String>,
    pub content: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CnappVulnerabilityRecord {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub cve_id: Option<String>,
    pub title: String,
    pub severity: String,
    pub resource_id: Option<String>,
    pub cloud_provider: Option<String>,
    pub status: String,
    pub discovered_at: String,
    pub content: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CnappAttackPathRecord {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub name: String,
    pub severity: String,
    pub path_length: i64,
    pub entry_point: Option<String>,
    pub target_asset: Option<String>,
    pub status: String,
    pub content: serde_json::Value,
    pub discovered_at: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct CnappFleetMonitor {
    pool: DbPool,
}

impl CnappFleetMonitor {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn record_rollup(
        &self,
        tenant_id: &str,
        controller_id: Option<&str>,
        payload: &CnappRollupPayload,
    ) -> Result<CnappFleetRollup, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let rollup_json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO tenant_cnapp_analytics_rollups (
                id, tenant_id, controller_id, reporting_accounts, posture_score, compliance_pct,
                open_vulnerabilities, critical_vulnerabilities, attack_paths_detected,
                multi_cloud_providers, fleet_risk_score, rollup_json, rolled_up_at, created_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(controller_id)
        .bind(payload.reporting_accounts)
        .bind(payload.posture_score)
        .bind(payload.compliance_pct)
        .bind(payload.open_vulnerabilities)
        .bind(payload.critical_vulnerabilities)
        .bind(payload.attack_paths_detected)
        .bind(payload.multi_cloud_providers)
        .bind(payload.fleet_risk_score)
        .bind(&rollup_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(CnappFleetRollup {
            id,
            tenant_id: tenant_id.to_string(),
            controller_id: controller_id.map(str::to_string),
            reporting_accounts: payload.reporting_accounts,
            posture_score: payload.posture_score,
            compliance_pct: payload.compliance_pct,
            open_vulnerabilities: payload.open_vulnerabilities,
            critical_vulnerabilities: payload.critical_vulnerabilities,
            attack_paths_detected: payload.attack_paths_detected,
            multi_cloud_providers: payload.multi_cloud_providers,
            fleet_risk_score: payload.fleet_risk_score,
            rollup: serde_json::from_str(&rollup_json).unwrap_or(serde_json::json!({})),
            rolled_up_at: now.clone(),
            created_at: now,
        })
    }

    pub async fn fleet_overview(&self, tenant_id: &str) -> Result<CnappFleetOverview, DbError> {
        let rollups = self.list_rollups(tenant_id, Some(50)).await?;
        let attack_paths = self.list_attack_paths(tenant_id, Some(50)).await?;
        let controllers_reporting = rollups
            .iter()
            .filter_map(|r| r.controller_id.as_deref())
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        let posture_score = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.posture_score).sum::<f64>() / rollups.len() as f64
        };

        let compliance_pct = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.compliance_pct).sum::<f64>() / rollups.len() as f64
        };

        let fleet_risk_score = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.fleet_risk_score).sum::<f64>() / rollups.len() as f64
        };

        Ok(CnappFleetOverview {
            tenant_id: tenant_id.to_string(),
            reporting_accounts: rollups.iter().map(|r| r.reporting_accounts).sum(),
            posture_score,
            compliance_pct,
            open_vulnerabilities: rollups.iter().map(|r| r.open_vulnerabilities).sum(),
            critical_vulnerabilities: rollups.iter().map(|r| r.critical_vulnerabilities).sum(),
            attack_paths_detected: rollups.iter().map(|r| r.attack_paths_detected).sum(),
            multi_cloud_providers: rollups
                .iter()
                .map(|r| r.multi_cloud_providers)
                .max()
                .unwrap_or(0),
            fleet_risk_score,
            controllers_reporting,
            rollups,
            attack_paths,
        })
    }

    pub async fn list_posture(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<CnappPostureRecord>, DbError> {
        let limit = limit.unwrap_or(100);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            Option<String>,
            String,
            f64,
            String,
            i64,
            String,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, cloud_provider, account_id, resource_kind,
                    posture_score, risk_level, findings_count, content_json, assessed_at,
                    created_at, updated_at
             FROM tenant_cnapp_posture WHERE tenant_id = ? ORDER BY assessed_at DESC LIMIT ?",
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
                    cloud_provider,
                    account_id,
                    resource_kind,
                    posture_score,
                    risk_level,
                    findings_count,
                    content_json,
                    assessed_at,
                    created_at,
                    updated_at,
                )| {
                    CnappPostureRecord {
                        id,
                        tenant_id,
                        controller_id,
                        cloud_provider,
                        account_id,
                        resource_kind,
                        posture_score,
                        risk_level,
                        findings_count,
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

    pub async fn list_compliance(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<CnappComplianceRecord>, DbError> {
        let limit = limit.unwrap_or(100);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            String,
            String,
            f64,
            Option<String>,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, framework, control_id, control_name, status,
                    compliance_pct, last_checked_at, content_json, created_at, updated_at
             FROM tenant_cnapp_compliance WHERE tenant_id = ? ORDER BY framework, control_id LIMIT ?",
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
                    framework,
                    control_id,
                    control_name,
                    status,
                    compliance_pct,
                    last_checked_at,
                    content_json,
                    created_at,
                    updated_at,
                )| {
                    CnappComplianceRecord {
                        id,
                        tenant_id,
                        controller_id,
                        framework,
                        control_id,
                        control_name,
                        status,
                        compliance_pct,
                        last_checked_at,
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn list_vulnerabilities(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<CnappVulnerabilityRecord>, DbError> {
        let limit = limit.unwrap_or(100);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, cve_id, title, severity, resource_id,
                    cloud_provider, status, discovered_at, content_json, created_at
             FROM tenant_cnapp_vulnerabilities WHERE tenant_id = ? ORDER BY discovered_at DESC LIMIT ?",
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
                    cve_id,
                    title,
                    severity,
                    resource_id,
                    cloud_provider,
                    status,
                    discovered_at,
                    content_json,
                    created_at,
                )| {
                    CnappVulnerabilityRecord {
                        id,
                        tenant_id,
                        controller_id,
                        cve_id,
                        title,
                        severity,
                        resource_id,
                        cloud_provider,
                        status,
                        discovered_at,
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        created_at,
                    }
                },
            )
            .collect())
    }

    pub async fn list_attack_paths(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<CnappAttackPathRecord>, DbError> {
        let limit = limit.unwrap_or(100);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            i64,
            Option<String>,
            Option<String>,
            String,
            String,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, name, severity, path_length, entry_point,
                    target_asset, status, content_json, discovered_at, created_at, updated_at
             FROM tenant_cnapp_attack_paths WHERE tenant_id = ? ORDER BY discovered_at DESC LIMIT ?",
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
                    severity,
                    path_length,
                    entry_point,
                    target_asset,
                    status,
                    content_json,
                    discovered_at,
                    created_at,
                    updated_at,
                )| {
                    CnappAttackPathRecord {
                        id,
                        tenant_id,
                        controller_id,
                        name,
                        severity,
                        path_length,
                        entry_point,
                        target_asset,
                        status,
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        discovered_at,
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
    ) -> Result<Vec<CnappFleetRollup>, DbError> {
        let limit = limit.unwrap_or(50);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            i64,
            f64,
            f64,
            i64,
            i64,
            i64,
            i64,
            f64,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, reporting_accounts, posture_score, compliance_pct,
                    open_vulnerabilities, critical_vulnerabilities, attack_paths_detected,
                    multi_cloud_providers, fleet_risk_score, rollup_json, rolled_up_at, created_at
             FROM tenant_cnapp_analytics_rollups WHERE tenant_id = ? ORDER BY rolled_up_at DESC LIMIT ?",
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
                    reporting_accounts,
                    posture_score,
                    compliance_pct,
                    open_vulnerabilities,
                    critical_vulnerabilities,
                    attack_paths_detected,
                    multi_cloud_providers,
                    fleet_risk_score,
                    rollup_json,
                    rolled_up_at,
                    created_at,
                )| {
                    CnappFleetRollup {
                        id,
                        tenant_id,
                        controller_id,
                        reporting_accounts,
                        posture_score,
                        compliance_pct,
                        open_vulnerabilities,
                        critical_vulnerabilities,
                        attack_paths_detected,
                        multi_cloud_providers,
                        fleet_risk_score,
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
