use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Duration;

use ia_sandbox::config::{
    ClearUsage, Config, Environment, Interactive, Limits, Mount, ShareNet, SpaceUsage,
    SwapRedirects,
};
use ia_sandbox::run_info::RunInfo;
use ia_sandbox::{self, Result};

pub struct ConfigBuilder {
    command: PathBuf,
    args: Vec<OsString>,
    new_root: Option<PathBuf>,
    share_net: bool,
    redirect_stdin: Option<PathBuf>,
    redirect_stdout: Option<PathBuf>,
    redirect_stderr: Option<PathBuf>,
    limits: Option<Limits>,
    instance_name: Option<OsString>,
    mounts: Vec<Mount>,
}

impl ConfigBuilder {
    pub fn new<T: AsRef<OsStr>>(command: T) -> ConfigBuilder {
        ConfigBuilder {
            command: command.as_ref().into(),
            args: vec![],
            new_root: None,
            share_net: true,
            redirect_stdin: Some("/dev/null".into()),
            redirect_stdout: Some("/dev/null".into()),
            redirect_stderr: Some("/dev/null".into()),
            limits: None,
            instance_name: Some("test".into()),
            mounts: vec![],
        }
    }

    pub fn command<T: AsRef<Path>>(&mut self, command: T) -> &mut ConfigBuilder {
        self.command = command.as_ref().into();
        self
    }

    pub fn arg<T: AsRef<OsStr>>(&mut self, arg: T) -> &mut ConfigBuilder {
        self.args.push(arg.as_ref().into());
        self
    }

    pub fn args<I, T>(&mut self, args: I) -> &mut ConfigBuilder
    where
        I: IntoIterator<Item = T>,
        T: AsRef<OsStr>,
    {
        for arg in args {
            self.arg(arg);
        }

        self
    }

    pub fn new_root<T: AsRef<Path>>(&mut self, new_root: T) -> &mut ConfigBuilder {
        self.new_root = Some(new_root.as_ref().into());
        self
    }

    pub fn share_net(&mut self, share_net: bool) -> &mut ConfigBuilder {
        self.share_net = share_net;
        self
    }

    pub fn stdin<T: AsRef<Path>>(&mut self, redirect_stdin: T) -> &mut ConfigBuilder {
        self.redirect_stdin = Some(redirect_stdin.as_ref().into());
        self
    }

    pub fn stdout<T: AsRef<Path>>(&mut self, redirect_stdin: T) -> &mut ConfigBuilder {
        self.redirect_stdout = Some(redirect_stdin.as_ref().into());
        self
    }

    pub fn stderr<T: AsRef<Path>>(&mut self, redirect_stdin: T) -> &mut ConfigBuilder {
        self.redirect_stderr = Some(redirect_stdin.as_ref().into());
        self
    }

    pub fn limits<T: Into<Limits>>(&mut self, limits: T) -> &mut ConfigBuilder {
        self.limits = Some(limits.into());
        self
    }

    pub fn instance_name<T: AsRef<OsStr>>(&mut self, instance_name: T) -> &mut ConfigBuilder {
        self.instance_name = Some(instance_name.as_ref().into());
        self
    }

    pub fn mount(&mut self, mount: Mount) -> &mut ConfigBuilder {
        self.mounts.push(mount);
        self
    }

    pub fn build_and_run(&mut self) -> Result<RunInfo<()>> {
        let config = Config::new(
            self.command.clone(),
            self.args.clone(),
            self.new_root.clone(),
            if self.share_net {
                ShareNet::Share
            } else {
                ShareNet::Unshare
            },
            self.redirect_stdin.clone(),
            self.redirect_stdout.clone(),
            self.redirect_stderr.clone(),
            self.limits.unwrap_or_default(),
            self.instance_name.clone(),
            Default::default(),
            self.mounts.clone(),
            SwapRedirects::No,
            ClearUsage::Yes,
            Interactive::No,
            Environment::EnvList(Vec::new()),
        );

        ia_sandbox::run_jail(&config)
    }
}

#[derive(Clone, Copy, Default)]
pub struct LimitsBuilder {
    wall_time: Option<Duration>,
    user_time: Option<Duration>,
    memory: Option<SpaceUsage>,
    stack: Option<SpaceUsage>,
    pids: Option<usize>,
}

impl LimitsBuilder {
    pub fn new() -> Self {
        LimitsBuilder {
            wall_time: None,
            user_time: None,
            memory: None,
            stack: None,
            pids: None,
        }
    }

    pub fn wall_time(&mut self, wall_time: Duration) -> &mut LimitsBuilder {
        self.wall_time = Some(wall_time);
        self
    }

    pub fn user_time(&mut self, user_time: Duration) -> &mut LimitsBuilder {
        self.user_time = Some(user_time);
        self
    }

    pub fn memory(&mut self, memory: SpaceUsage) -> &mut LimitsBuilder {
        self.memory = Some(memory);
        self
    }

    pub fn stack(&mut self, stack: SpaceUsage) -> &mut LimitsBuilder {
        self.stack = Some(stack);
        self
    }

    pub fn pids(&mut self, pids: usize) -> &mut LimitsBuilder {
        self.pids = Some(pids);
        self
    }
}

impl From<LimitsBuilder> for Limits {
    fn from(limits_builder: LimitsBuilder) -> Limits {
        Limits::new(
            limits_builder.wall_time,
            limits_builder.user_time,
            limits_builder.memory,
            limits_builder.stack,
            limits_builder.pids,
        )
    }
}
