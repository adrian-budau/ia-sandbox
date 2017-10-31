ia-sandbox
==========
Infoarena sandbox for running user submitted code, in rust using namespaces and cgroups.

[![TravisCI](https://travis-ci.org/adrian-budau/ia-sandbox.svg?branch=master)](https://travis-ci.org/adrian-budau/io-sandbox)
[![Code Coverage](https://codecov.io/gh/adrian-budau/ia-sandbox/branch/master/graph/badge.svg)](https://codecov.io/gh/adrian-budau/ia-sandbox)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Currently a work in progress.

What is done
------------

- [x] Namespacess (mount, pid, ipc, uts, user)
- [x] Pivot root 
    - [x] Proper unmounting of all previous mounts
- [x] Redirect of stdin/stdout/stderr
- [ ] Memory/Disk/Cpu limits
    - [ ] Setrlimit
    - [ ] Cgroups
- [ ] Collect run data
    - [x] Collect exit status (success, non zero exit status, killed by signal)
    - [ ] Collect memory/time/disk usage
