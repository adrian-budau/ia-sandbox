# ia-sandbox
Infoarena sandbox for running user submitted code, in rust using namespaces and cgroups.

Currently a work in progress.

# What is done

- [x] Namespacess (mount, pid, ipc, uts, user)
- [x] Pivot root 
    - [x] Proper unmounting of all previous mounts
- [ ] Redirect of stdin/stdout/stderr
- [ ] Memory/Disk/Cpu limits
    - [ ] Setrlimit
    - [ ] Cgroups
- [ ] Collect run data
