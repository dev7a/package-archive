//! This module provides utilities for configuring and building an OpenTelemetry MeterProvider
//! specifically tailored for use in AWS Lambda environments.
//!
//! It includes:
//! - `HttpMeterProviderBuilder`: A builder struct for configuring and initializing a MeterProvider.
//! 
//! The module supports various configuration options, including:
//! - Custom HTTP clients for exporting metrics
//! - Setting custom meter names
//! - Configuring periodic exporters
//! - Integration with Lambda resource attributes

use std::time::Duration;

use opentelemetry::metrics::{MeterProvider, Result as MetricsResult};
use opentelemetry_http::HttpClient;
use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig};
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider, InstrumentKind},
    metrics::data::Temporality,
    metrics::reader::TemporalitySelector,
    runtime,
};
use otlp_stdout_client::StdoutClient;
use std::env;
use crate::http_tracer_provider::get_lambda_resource;

/// Default temporality selector that uses Delta for all instruments
#[derive(Debug, Default)]
struct DefaultTemporalitySelector;

impl TemporalitySelector for DefaultTemporalitySelector {
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        match kind {
            InstrumentKind::Counter | InstrumentKind::ObservableCounter | InstrumentKind::Histogram => {
                Temporality::Delta
            }
            InstrumentKind::UpDownCounter | InstrumentKind::ObservableUpDownCounter | InstrumentKind::ObservableGauge | InstrumentKind::Gauge => {
                Temporality::Cumulative
            }
        }
    }
}

/// Builder for configuring and initializing a MeterProvider.
///
/// This struct provides a fluent interface for configuring various aspects of the
/// OpenTelemetry metrics setup, including the exporter configuration and meter names.
///
/// # Examples
///
/// ```
/// use lambda_otel_utils::HttpMeterProviderBuilder;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), opentelemetry::metrics::MetricsError> {
///     let meter_provider = HttpMeterProviderBuilder::default()
///         .with_stdout_client()
///         .with_meter_name("my-service")
///         .with_export_interval(Duration::from_secs(60))
///         .build()?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct HttpMeterProviderBuilder<C: HttpClient + 'static = StdoutClient> {
    client: Option<C>,
    meter_name: Option<&'static str>,
    export_interval: Duration,
    export_timeout: Duration,
}

impl Default for HttpMeterProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpMeterProviderBuilder {
    /// Creates a new `HttpMeterProviderBuilder` with default settings.
    pub fn new() -> Self {
        Self {
            client: None,
            meter_name: None,
            export_interval: Duration::from_secs(60),
            export_timeout: Duration::from_secs(10),
        }
    }

    /// Configures the builder to use a stdout client for exporting metrics.
    pub fn with_stdout_client(mut self) -> Self {
        self.client = Some(StdoutClient::new());
        self
    }

    /// Sets the meter name.
    ///
    /// # Arguments
    ///
    /// * `meter_name` - A static string reference (string literal)
    pub fn with_meter_name(mut self, meter_name: &'static str) -> Self {
        self.meter_name = Some(meter_name);
        self
    }

    /// Sets the export interval for periodic metric collection.
    pub fn with_export_interval(mut self, interval: Duration) -> Self {
        self.export_interval = interval;
        self
    }

    /// Sets the export timeout for metric collection.
    pub fn with_export_timeout(mut self, timeout: Duration) -> Self {
        self.export_timeout = timeout;
        self
    }

    /// Builds the `MeterProvider` with the configured settings.
    pub fn build(self) -> MetricsResult<SdkMeterProvider> {
        let protocol = match env::var("OTEL_EXPORTER_OTLP_PROTOCOL")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "http/protobuf" => Protocol::HttpBinary,
            "http/json" | "" => Protocol::HttpJson,
            unsupported => {
                eprintln!(
                    "Warning: OTEL_EXPORTER_OTLP_PROTOCOL value '{}' is not supported. Defaulting to HTTP JSON.",
                    unsupported
                );
                Protocol::HttpJson
            }
        };

        let export_config = ExportConfig {
            protocol,
            timeout: self.export_timeout,
            ..Default::default()
        };

        let mut exporter_builder = opentelemetry_otlp::new_exporter().http().with_export_config(export_config);

        if let Some(client) = self.client {
            exporter_builder = exporter_builder.with_http_client(client);
        }

        // Build the metrics exporter with default temporality selector
        let exporter = exporter_builder.build_metrics_exporter(Box::new(DefaultTemporalitySelector))?;

        let reader = PeriodicReader::builder(exporter, runtime::Tokio)
            .with_interval(self.export_interval)
            .build();

        let provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_resource(get_lambda_resource())
            .build();

        Ok(provider)
    }

    /// Gets a meter from the provider with the configured name.
    pub fn get_meter(self, provider: &SdkMeterProvider) -> opentelemetry::metrics::Meter {
        provider.meter(self.meter_name.unwrap_or("default"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_meter_provider_builder_default() {
        let builder = HttpMeterProviderBuilder::default();
        assert!(builder.client.is_none());
        assert!(builder.meter_name.is_none());
        assert_eq!(builder.export_interval, Duration::from_secs(60));
        assert_eq!(builder.export_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_http_meter_provider_builder_customization() {
        let builder = HttpMeterProviderBuilder::new()
            .with_stdout_client()
            .with_meter_name("test-meter")
            .with_export_interval(Duration::from_secs(30))
            .with_export_timeout(Duration::from_secs(5));

        assert!(builder.client.is_some());
        assert_eq!(builder.meter_name, Some("test-meter"));
        assert_eq!(builder.export_interval, Duration::from_secs(30));
        assert_eq!(builder.export_timeout, Duration::from_secs(5));
    }
} 