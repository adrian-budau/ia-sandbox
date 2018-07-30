# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2018-07-30
### Added
- A basic sandbox using linux namespaces and the cgroups subsystem for limits
  and isolation.
- Integration with gitlab CI and testing on all 3 toolchains:
  stable, beta and nightly.
- Redirection of standard input/output/error to the isolated application.
- Tests.
- Resource usage (wall time, user time, memory) and exit status
  (success, killed by signal, non-zero exit status) collection.
- Fork bomb protection.
- Mounting external folders inside the mount namespace (make it available to
  the sandboxed application).
- The ability to reverse the opening of input/output, useful for running
  interactive tasks using standard input/output.
- The ability to not reclear usage on start, useful for multi-run applications.
