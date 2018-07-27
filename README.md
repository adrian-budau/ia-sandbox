ia-sandbox
==========
Infoarena sandbox for running user submitted code, in rust using namespaces and cgroups.

[![TravisCI](https://travis-ci.org/adrian-budau/ia-sandbox.svg?branch=master)](https://travis-ci.org/adrian-budau/ia-sandbox)
[![Code Coverage](https://codecov.io/gh/adrian-budau/ia-sandbox/branch/master/graph/badge.svg)](https://codecov.io/gh/adrian-budau/ia-sandbox)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Currently a work in progress.

What is done
------------

- [x] Namespacess (mount, pid, ipc, uts, user)
- [x] Pivot root 
    - [x] Proper unmounting of all previous mounts
- [x] Redirect of stdin/stdout/stderr
- [x] Memory/Disk/Cpu limits using cGroups
- [x] Collect run data
    - [x] Collect exit status (success, non zero exit status, killed by signal)
    - [x] Collect memory/time/disk usage
    - [x] Output in a human readable format
    - [x] Output old jail line from infoarena
    - [x] Output json
- [x] Support interactive tasks (using pipes)
- [x] Support multi-run tasks (not resetting stats)
