use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::pipeline::TelemetryRecord;

#[async_trait]
pub trait TelemetryExporter: Send + Sync {
    fn name(&self) -> &'static str;
    async fn export(&self, records: &[TelemetryRecord]) -> Result<(), String>;
}

pub struct JsonMetricsExporter;

#[async_trait]
impl TelemetryExporter for JsonMetricsExporter {
    fn name(&self) -> &'static str {
        "json"
    }

    async fn export(&self, records: &[TelemetryRecord]) -> Result<(), String> {
        let _ = serde_json::to_string(records).map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub struct PrometheusExporter;

#[async_trait]
impl TelemetryExporter for PrometheusExporter {
    fn name(&self) -> &'static str {
        "prometheus"
    }

    async fn export(&self, records: &[TelemetryRecord]) -> Result<(), String> {
        let mut lines = String::new();
        for record in records {
            let name = sanitize_metric_name(&record.name);
            lines.push_str(&format!(
                "# TYPE {name} gauge\n{name} {} {}\n",
                record.value,
                format_labels(&record.labels)
            ));
        }
        let _ = lines;
        Ok(())
    }
}

fn sanitize_metric_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn format_labels(labels: &[(String, String)]) -> String {
    if labels.is_empty() {
        return String::new();
    }
    let body = labels
        .iter()
        .map(|(k, v)| format!("{k}=\"{v}\""))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{body}}}")
}

pub struct OtlpExporter {
    endpoint: String,
}

impl OtlpExporter {
    pub fn from_env() -> Self {
        Self {
            endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".into()),
        }
    }
}

#[async_trait]
impl TelemetryExporter for OtlpExporter {
    fn name(&self) -> &'static str {
        "otlp"
    }

    async fn export(&self, records: &[TelemetryRecord]) -> Result<(), String> {
        #[cfg(feature = "otel")]
        {
            use opentelemetry::global;
            use opentelemetry::metrics::{Counter, Meter};

            let meter = global::meter("wiresentinel-cloud");
            for record in records {
                let counter: Counter<f64> = meter
                    .f64_counter(sanitize_metric_name(&record.name))
                    .build();
                counter.add(record.value, &[]);
            }
            let _ = &self.endpoint;
            return Ok(());
        }

        #[cfg(not(feature = "otel"))]
        {
            let _ = (&self.endpoint, records);
            tracing::debug!("OTEL disabled; stub export of {} records", records.len());
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExporterStatus {
    pub name: String,
    pub enabled: bool,
    pub stub: bool,
}

impl ExporterStatus {
    pub fn otlp_status() -> Self {
        Self {
            name: "otlp".into(),
            enabled: std::env::var("OTEL_ENABLED").ok().as_deref() == Some("1"),
            stub: !cfg!(feature = "otel"),
        }
    }
}
