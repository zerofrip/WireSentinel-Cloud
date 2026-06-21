use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XdrRollupPayload {
    pub reporting_devices: i64,
    pub total_incidents: i64,
    pub open_incidents: i64,
    pub critical_incidents: i64,
    pub total_detections: i64,
    pub active_hunts: i64,
    pub mitre_techniques_detected: i64,
    pub mitre_coverage_pct: f64,
    pub avg_incident_mttr_hours: f64,
    pub fleet_threat_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XdrFleetRollup {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub reporting_devices: i64,
    pub total_incidents: i64,
    pub open_incidents: i64,
    pub critical_incidents: i64,
    pub total_detections: i64,
    pub active_hunts: i64,
    pub mitre_techniques_detected: i64,
    pub mitre_coverage_pct: f64,
    pub avg_incident_mttr_hours: f64,
    pub fleet_threat_score: f64,
    pub rollup: serde_json::Value,
    pub rolled_up_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XdrFleetOverview {
    pub tenant_id: String,
    pub reporting_devices: i64,
    pub total_incidents: i64,
    pub open_incidents: i64,
    pub critical_incidents: i64,
    pub total_detections: i64,
    pub active_hunts: i64,
    pub mitre_techniques_detected: i64,
    pub mitre_coverage_pct: f64,
    pub avg_incident_mttr_hours: f64,
    pub fleet_threat_score: f64,
    pub controllers_reporting: i64,
    pub rollups: Vec<XdrFleetRollup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XdrIncidentRecord {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub title: String,
    pub status: String,
    pub severity: String,
    pub detection_count: i64,
    pub content: serde_json::Value,
    pub opened_at: String,
    pub resolved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XdrDetectionRecord {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub rule_name: String,
    pub rule_kind: String,
    pub severity: String,
    pub mitre_technique_id: Option<String>,
    pub device_id: Option<String>,
    pub matched_at: String,
    pub content: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XdrMitreCoverageRecord {
    pub id: String,
    pub tenant_id: String,
    pub tactic: String,
    pub technique_id: String,
    pub technique_name: String,
    pub detection_count: i64,
    pub coverage_pct: f64,
    pub last_seen_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct XdrFleetMonitor {
    pool: DbPool,
}

impl XdrFleetMonitor {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn record_rollup(
        &self,
        tenant_id: &str,
        controller_id: Option<&str>,
        payload: &XdrRollupPayload,
    ) -> Result<XdrFleetRollup, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let rollup_json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO tenant_xdr_analytics_rollups (
                id, tenant_id, controller_id, reporting_devices, total_incidents, open_incidents,
                critical_incidents, total_detections, active_hunts, mitre_techniques_detected,
                mitre_coverage_pct, avg_incident_mttr_hours, fleet_threat_score,
                rollup_json, rolled_up_at, created_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(controller_id)
        .bind(payload.reporting_devices)
        .bind(payload.total_incidents)
        .bind(payload.open_incidents)
        .bind(payload.critical_incidents)
        .bind(payload.total_detections)
        .bind(payload.active_hunts)
        .bind(payload.mitre_techniques_detected)
        .bind(payload.mitre_coverage_pct)
        .bind(payload.avg_incident_mttr_hours)
        .bind(payload.fleet_threat_score)
        .bind(&rollup_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(XdrFleetRollup {
            id,
            tenant_id: tenant_id.to_string(),
            controller_id: controller_id.map(str::to_string),
            reporting_devices: payload.reporting_devices,
            total_incidents: payload.total_incidents,
            open_incidents: payload.open_incidents,
            critical_incidents: payload.critical_incidents,
            total_detections: payload.total_detections,
            active_hunts: payload.active_hunts,
            mitre_techniques_detected: payload.mitre_techniques_detected,
            mitre_coverage_pct: payload.mitre_coverage_pct,
            avg_incident_mttr_hours: payload.avg_incident_mttr_hours,
            fleet_threat_score: payload.fleet_threat_score,
            rollup: serde_json::from_str(&rollup_json).unwrap_or(serde_json::json!({})),
            rolled_up_at: now.clone(),
            created_at: now,
        })
    }

    pub async fn fleet_overview(&self, tenant_id: &str) -> Result<XdrFleetOverview, DbError> {
        let rollups = self.list_rollups(tenant_id, Some(50)).await?;
        let controllers_reporting = rollups
            .iter()
            .filter_map(|r| r.controller_id.as_deref())
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        let mitre_coverage_pct = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.mitre_coverage_pct).sum::<f64>() / rollups.len() as f64
        };

        let avg_incident_mttr_hours = if rollups.is_empty() {
            0.0
        } else {
            rollups
                .iter()
                .map(|r| r.avg_incident_mttr_hours)
                .sum::<f64>()
                / rollups.len() as f64
        };

        let fleet_threat_score = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.fleet_threat_score).sum::<f64>() / rollups.len() as f64
        };

        Ok(XdrFleetOverview {
            tenant_id: tenant_id.to_string(),
            reporting_devices: rollups.iter().map(|r| r.reporting_devices).sum(),
            total_incidents: rollups.iter().map(|r| r.total_incidents).sum(),
            open_incidents: rollups.iter().map(|r| r.open_incidents).sum(),
            critical_incidents: rollups.iter().map(|r| r.critical_incidents).sum(),
            total_detections: rollups.iter().map(|r| r.total_detections).sum(),
            active_hunts: rollups.iter().map(|r| r.active_hunts).sum(),
            mitre_techniques_detected: rollups
                .iter()
                .map(|r| r.mitre_techniques_detected)
                .sum(),
            mitre_coverage_pct,
            avg_incident_mttr_hours,
            fleet_threat_score,
            controllers_reporting,
            rollups,
        })
    }

    pub async fn list_incidents(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<XdrIncidentRecord>, DbError> {
        let limit = limit.unwrap_or(100);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            String,
            i64,
            String,
            String,
            Option<String>,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, title, status, severity, detection_count,
                    content_json, opened_at, resolved_at, created_at, updated_at
             FROM tenant_xdr_incidents WHERE tenant_id = ? ORDER BY opened_at DESC LIMIT ?",
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
                    detection_count,
                    content_json,
                    opened_at,
                    resolved_at,
                    created_at,
                    updated_at,
                )| {
                    XdrIncidentRecord {
                        id,
                        tenant_id,
                        controller_id,
                        title,
                        status,
                        severity,
                        detection_count,
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

    pub async fn list_detections(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<XdrDetectionRecord>, DbError> {
        let limit = limit.unwrap_or(100);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, rule_name, rule_kind, severity,
                    mitre_technique_id, device_id, matched_at, content_json, created_at
             FROM tenant_xdr_detections WHERE tenant_id = ? ORDER BY matched_at DESC LIMIT ?",
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
                    rule_name,
                    rule_kind,
                    severity,
                    mitre_technique_id,
                    device_id,
                    matched_at,
                    content_json,
                    created_at,
                )| {
                    XdrDetectionRecord {
                        id,
                        tenant_id,
                        controller_id,
                        rule_name,
                        rule_kind,
                        severity,
                        mitre_technique_id,
                        device_id,
                        matched_at,
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        created_at,
                    }
                },
            )
            .collect())
    }

    pub async fn list_mitre_coverage(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<XdrMitreCoverageRecord>, DbError> {
        let rows: Vec<(
            String,
            String,
            String,
            String,
            String,
            i64,
            f64,
            Option<String>,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, tactic, technique_id, technique_name, detection_count,
                    coverage_pct, last_seen_at, created_at, updated_at
             FROM tenant_xdr_mitre_coverage WHERE tenant_id = ? ORDER BY tactic, technique_id",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    tenant_id,
                    tactic,
                    technique_id,
                    technique_name,
                    detection_count,
                    coverage_pct,
                    last_seen_at,
                    created_at,
                    updated_at,
                )| {
                    XdrMitreCoverageRecord {
                        id,
                        tenant_id,
                        tactic,
                        technique_id,
                        technique_name,
                        detection_count,
                        coverage_pct,
                        last_seen_at,
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
    ) -> Result<Vec<XdrFleetRollup>, DbError> {
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
            i64,
            i64,
            f64,
            f64,
            f64,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, reporting_devices, total_incidents, open_incidents,
                    critical_incidents, total_detections, active_hunts, mitre_techniques_detected,
                    mitre_coverage_pct, avg_incident_mttr_hours, fleet_threat_score,
                    rollup_json, rolled_up_at, created_at
             FROM tenant_xdr_analytics_rollups WHERE tenant_id = ? ORDER BY rolled_up_at DESC LIMIT ?",
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
                    reporting_devices,
                    total_incidents,
                    open_incidents,
                    critical_incidents,
                    total_detections,
                    active_hunts,
                    mitre_techniques_detected,
                    mitre_coverage_pct,
                    avg_incident_mttr_hours,
                    fleet_threat_score,
                    rollup_json,
                    rolled_up_at,
                    created_at,
                )| {
                    XdrFleetRollup {
                        id,
                        tenant_id,
                        controller_id,
                        reporting_devices,
                        total_incidents,
                        open_incidents,
                        critical_incidents,
                        total_detections,
                        active_hunts,
                        mitre_techniques_detected,
                        mitre_coverage_pct,
                        avg_incident_mttr_hours,
                        fleet_threat_score,
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
