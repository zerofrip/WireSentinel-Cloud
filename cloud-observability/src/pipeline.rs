use std::sync::Arc;

use cloud_core::CloudMetricsAggregator;
use database::DbPool;
use serde::{Deserialize, Serialize};

use crate::exporters::{
    ExporterStatus, JsonMetricsExporter, OtlpExporter, PrometheusExporter, TelemetryExporter,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryRecord {
    pub name: String,
    pub value: f64,
    pub labels: Vec<(String, String)>,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub otel_enabled: bool,
    pub export_interval_secs: u64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            otel_enabled: std::env::var("OTEL_ENABLED").ok().as_deref() == Some("1"),
            export_interval_secs: 60,
        }
    }
}

pub struct TelemetryPipeline {
    pool: DbPool,
    metrics: CloudMetricsAggregator,
    exporters: Vec<Arc<dyn TelemetryExporter>>,
    config: TelemetryConfig,
}

impl TelemetryPipeline {
    pub fn new(pool: DbPool) -> Self {
        let metrics = CloudMetricsAggregator::new(pool.clone());
        let mut exporters: Vec<Arc<dyn TelemetryExporter>> = vec![
            Arc::new(PrometheusExporter),
            Arc::new(JsonMetricsExporter),
        ];
        if std::env::var("OTEL_ENABLED").ok().as_deref() == Some("1") {
            exporters.push(Arc::new(OtlpExporter::from_env()));
        }
        Self {
            pool,
            metrics,
            exporters,
            config: TelemetryConfig::default(),
        }
    }

    pub fn with_config(pool: DbPool, config: TelemetryConfig) -> Self {
        let mut pipeline = Self::new(pool);
        pipeline.config = config;
        pipeline
    }

    pub fn exporter_status(&self) -> Vec<ExporterStatus> {
        let mut statuses = vec![
            ExporterStatus {
                name: "prometheus".into(),
                enabled: true,
                stub: false,
            },
            ExporterStatus {
                name: "json".into(),
                enabled: true,
                stub: false,
            },
            ExporterStatus::otlp_status(),
        ];
        if !self.config.otel_enabled {
            if let Some(otlp) = statuses.iter_mut().find(|s| s.name == "otlp") {
                otlp.enabled = false;
            }
        }
        statuses
    }

    pub async fn collect_records(&self) -> Result<Vec<TelemetryRecord>, database::DbError> {
        let snapshot = self.metrics.snapshot().await?;
        let ts = database::models::now_iso();
        Ok(vec![
            TelemetryRecord {
                name: "ws_cloud_tenants_active".into(),
                value: snapshot.tenants_active as f64,
                labels: vec![],
                timestamp: ts.clone(),
            },
            TelemetryRecord {
                name: "ws_cloud_organizations_total".into(),
                value: snapshot.organizations_total as f64,
                labels: vec![],
                timestamp: ts.clone(),
            },
            TelemetryRecord {
                name: "ws_cloud_federated_controllers_total".into(),
                value: snapshot.federated_controllers_total as f64,
                labels: vec![],
                timestamp: ts.clone(),
            },
            TelemetryRecord {
                name: "ws_cloud_uptime_seconds".into(),
                value: snapshot.uptime_seconds as f64,
                labels: vec![],
                timestamp: ts,
            },
        ])
    }

    pub async fn flush(&self) -> Result<FlushReport, String> {
        let records = self
            .collect_records()
            .await
            .map_err(|e| e.to_string())?;
        let mut exported = Vec::new();
        for exporter in &self.exporters {
            if exporter.name() == "otlp" && !self.config.otel_enabled {
                continue;
            }
            exporter.export(&records).await?;
            exported.push(exporter.name().to_string());
        }
        Ok(FlushReport {
            record_count: records.len(),
            exporters: exported,
        })
    }

    pub async fn tenant_bandwidth_bytes(&self, tenant_id: &str) -> Result<f64, database::DbError> {
        let row: Option<(f64,)> = sqlx::query_as(
            "SELECT COALESCE(SUM(total), 0) FROM usage_aggregates WHERE tenant_id = ? AND metric = 'bandwidth_bytes'",
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0).unwrap_or(0.0))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlushReport {
    pub record_count: usize,
    pub exporters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityMetricsResponse {
    pub records: Vec<TelemetryRecord>,
    pub exporters: Vec<ExporterStatus>,
    pub prometheus_text: String,
}

impl TelemetryPipeline {
    pub async fn observability_snapshot(
        &self,
    ) -> Result<ObservabilityMetricsResponse, database::DbError> {
        let records = self.collect_records().await?;
        let global = self.metrics.snapshot().await?;
        Ok(ObservabilityMetricsResponse {
            prometheus_text: CloudMetricsAggregator::to_prometheus(&global),
            exporters: self.exporter_status(),
            records,
        })
    }
}
