# Package Archive

This repository serves as a historical archive for packages that have been deprecated and are no longer actively maintained. These packages were previously published to package registries like NPM, PyPI, and crates.io.

## Purpose

This archive preserves the source code of deprecated packages while clearly marking them as deprecated both in their READMEs and in their respective package registries. This helps ensure that:

1. Users understand these packages are no longer maintained
2. The history and implementation details remain accessible
3. Users are directed to modern alternatives

## Deprecated Packages

### Python

- **[otlp-stdout-adapter](packages/python/adapter)**: A custom requests HTTPAdapter that serializes spans to stdout for OpenTelemetry OTLP exporters
  - Deprecated in favor of: [otlp-stdout-span-exporter](https://pypi.org/project/otlp-stdout-span-exporter/)

### Node.js

- **[@dev7a/otlp-stdout-exporter](packages/node/exporter)**: OpenTelemetry OTLP exporter that writes to stdout
  - Deprecated in favor of: [@dev7a/otlp-stdout-span-exporter](https://www.npmjs.com/package/@dev7a/otlp-stdout-span-exporter)

### Rust

- **[otlp-stdout-client](packages/rust/otlp-stdout-client)**: OpenTelemetry OTLP stdout client
  - Deprecated in favor of: [otlp-stdout-span-exporter](https://crates.io/crates/otlp-stdout-span-exporter)

- **[lambda-otel-utils](packages/rust/lambda-otel-utils)**: OpenTelemetry utilities for AWS Lambda
  - Deprecated in favor of: [lambda-otel-lite](https://crates.io/crates/lambda-otel-lite)

## Note on Replacements

The suggested replacement packages are not exact drop-in replacements as their APIs differ from the deprecated packages. However, they solve the same problems with improved, more modern implementations.

## License

All packages in this repository are licensed under the MIT License - see the individual LICENSE files within each package directory for details. 