# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2018-08-10
### Added
- Interactive tasks test (2 process communicating by stdin/stdout). Was also
  used to test `swap-redirects` is properly working.

### Changed
- `run_jail` is gone and `spawn_jail` has taken its place.

## [0.1.3] - 2018-08-10
### Changed
- stdin/stdout/stderr redirection is now using `dup` instead of assuming
  that `close` followed by `open` will assign the lowest fd.

## [0.1.2] - 2018-08-06
### Changed
- Checked the minimal versions of dependencies

## [0.1.1] - 2018-07-30
### Added
- Interactive mode (for easily landing into a shell).
- Environment variables forwarding and passing

### Changed
- Correct security with cgroup namespaces.

## [0.1.0] - 2018-07-30
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
