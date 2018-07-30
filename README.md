ia-sandbox
==========
Infoarena sandbox for running user submitted code, in rust using namespaces and cgroups.

[![pipeline status](https://gitlab.com/adrian.budau/ia-sandbox/badges/master/pipeline.svg)](https://gitlab.com/adrian.budau/ia-sandbox/commits/master)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Currently a work in progress and experimental, use it at your own risk.

### CHANGELOG

Please see the [CHANGELOG](CHANGELOG.md) for a release history.

### Quick Links

* [What is ia-sandbox?](#what-is-ia-sandbox)
* [Installation](#installation)
* [How does it work?](#how-does-it-work)
* [Contribuiting](#contribuiting)

### What is ia-sandbox?

ia-sandbox is a command line utility allowing you to run untrusted code, such as
the one submitted by users on websites like [Codeforces](https://codeforces.com),
[Topcoder](https://www.topcoder.com/community/competitive-programming/) or the
website it was designed for [Infoarena](https://infoarena.ro) with certain limits
for time (both user time and wall time), memory and number of processes while also
collecting runtime information and exit status.

It uses modern linux tools and as such requires a relatively new kernel:
* [cgroups v1](https://www.kernel.org/doc/Documentation/cgroup-v1/cgroups.txt) - a linux
  kernel feature for limiting, accounting and isolating resource usage of a collection
  of proceeses
  * __cpuacct__ - for precise user time usage and limits, requires a 64-bit computer for proper
    precision. Requires linux kernel &ge; __2.6.24__
  * __memory__ - for memory usage and limits. Requires linux kernel &ge; __2.6.24__
  * __pids__ - for limiting the number of processes, necessary for protection against
    fork bombs. Requires linux kernel &ge; __4.3__
* [linux namespaces](http://man7.org/linux/man-pages/man7/namespaces.7.html): - another linux
  kernel feature for isolating resources on the system
  * __mount__ - for isolating mountpoints, the isolated application will only see itself and
    whatever the caller desides to mount next to it. Requires linux kernel &ge; __2.4.19__
  * __ipc__ - for ipc isolation. Requires linux kernel &ge; __2.6.19__
  * __uts__ - for uts isolation. Requires linux kernel &ge; __2.6.19__
  * __pid__ - for isolating processes, the sandboxed application will see itself as
    the only running process on the entire system. Requires linux kernel &ge; __2.6.24__
  * __network__ - for isolating network interfaces, the application can not make any changes
    to network, or see external changes. Requires linux kernel &ge; __2.6.24__
  * __user namespaces__ - for isolating the user id and group id. The isolated application
    will see itself as root inside the sandbox, and as having all capabilities but
    for the purpose of accessing resources it will actually be the user id of sandbox
    owner, greatly limiting whatever power or exploit the application can do. Requires
    linux kernel &ge; __3.5__, but for proper security it is better for it to &ge; __3.9__
  * __cgroup__ - for isolating cgroups, to not allow the sandboxed application to change
    its own limits or usage. Requires linux kernel &ge; __4.6__

Since all of these features need to be active, the minimum required version is __4.6__.

### Installation

The binary name is `ia-sandbox`.

The easiest way to install this is using `cargo` the package manager for
[Rust](https://www.rust-lang.org/).
* The minimum supported version of Rust for `ia-sandbox` is __1.27.0__, altough
  `ia-sandbox` might work with older versions.

```
cargo install ia-sandbox
```

For actual isolation it is best to change the root of the sandbox (using `-r` or `--new-root`).
This will unmount everything, except for `/proc` which is necessary, and is already only
showing the isolated process.

If you would like to explore the inside of the sandbox an easy way would be

```
ia-sandbox --mount /lib:/lib:exec --mount /lib64:/lib64:exec --mount /usr:/usr:exec
           --mount /bin:/bin:exec --new-root PATH_TO_SOME_FOLDER --interactive --forward-env
           /bin/bash
```

### How does it work?

- It first spawns a `supervisor` process into a new pid and user namespaces (while
  also setting the right uid, gid and proc mount).
- It sets the supervisor to be killed when it's parent dies.
- It then spawns the process that will actually become sandboxed application. 
  - It does this to protect itself from the case where the parent process dies,
    because it was forcefully killed and there is no one to kill the sandboxed
    application should it exceed its limits. By pid namespaces design, if the
    init process in the namespace dies, all processes get killed.
  - This new process is spawned in completely different namespaces except for cgroup.
- It redirects standard input and output (while it still has acces to the file paths), if
  configured.
- It sets the stack limit.
  - This has nothing to do with security, but rather with providing as much stack
    memory as required for the process to run.
- It enters the cgroups necessary (cpuacct, memory, pids) optionally not clearing the
  usage from previous runs.
- It enters a new cgroup namespace.
- If a new root is requested (via `--new-root` or `-r`), it pivot roots to that path
- It mounts the `/proc` path.
- It sets the uid/gid map.
- It moves to a different process group.
- Lastly it execs the given application.

### Contribuiting.

For any issues, especially security-related please open an issue at [Issues](https://gitlab.com/adrian.budau/ia-sandbox/issues).

I can not make any promises about pull requests, but I am open to them :-).

If you want to run the tests you will need to build the test-fixtures first, this can be easily done with

```
cargo build --features integration-test --bins
```

and then run the tests

```
cargo test
```

There are fixtures that require nightly only features (such as inline assembly). If running on nightly you can compile
the fixtures with

```
cargo build --features integration-test,nightly --bins
```

and then run the test with

```
cargo test --features nightly
```
