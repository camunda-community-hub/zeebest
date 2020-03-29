# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
Project Abandoned.
### Postmortem
Things that went good: :tada:
- Learned a lot about Rust and Cargo 
- Experimented with cool tools: Zeebe, Protobufs
- Project vision was good

Things that went bad: :collision:
- Rust futures have been painful to work with  
- Lots of toil work in futures-runtime compatibility - zeebest should work on all runtimes
- Maintainer lost interest and works on different projects now

## [0.20.0] - 2019-09-21
Lots of big changes in this release! New futures and breaking API change.
### Changed
- the client and worker now use futures 0.3
- the worker api has been changed to return a unit future
- switch to zeebe 0.20 and change to their community license

## [0.18.1] - 2019-06-18
### Added
- A complete example app based on the [order-process workflow from Zeebe's "Getting Started Tutorial"][order_process].
- The job stream now returns the completed `ActivatedJob` struct instead of just the job key. 
### Changed
- `Sync + UnwindSafe` is now enforced on job handlers and all work is done on a threadpool with `futures-cpupool`.
- The `Client::worker` method now accepts `Duration` for timeout instead of an `i64` and there is an added assertion.

## [0.18.0] - 2019-06-10
Initial release of zeebest 🥏

[Unreleased]: https://github.com/xmclark/zeebest/compare/v0.20.0...HEAD
[0.20.0]: https://github.com/xmclark/zeebest/compare//v0.18.1..v0.20.0
[0.18.1]: https://github.com/xmclark/zeebest/compare//v0.18.0..v0.18.1
[0.18.0]: https://github.com/xmclark/zeebest/releases/tag/v0.18.0

[order_process]: https://docs.zeebe.io/getting-started/README.html