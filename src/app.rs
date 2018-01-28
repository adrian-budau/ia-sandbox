use clap::{App, AppSettings, Arg};

const ABOUT: &'static str = "
ia-sandbox sandboxes applications for secure running of executables";

const LONG_ABOUT: &'static str = "
ia-sandbox sandboxes applications for secure running of executables

ia-sandbox uses cgroups, namespaces, pivot_root and other techniques to
guarantee security. It is designed to be used for online judges and in
particular infoarena.ro";

const ARGS_AFTER_HELP: &'static str = "
All of the trailing arguments are passed to the command to run. If you're passing
arguments to both ia-sandbox and the binary, the ones after `--` go to the command,
the ones before go to ia-sandbox.";

pub fn app() -> App<'static, 'static> {
    App::new("ia-sandbox")
        .author(crate_authors!())
        .version(crate_version!())
        .about(ABOUT)
        .long_about(LONG_ABOUT)
        .max_term_width(100)
        .setting(AppSettings::UnifiedHelpMessage)
        .help_message("Prints help information. Use --help for more details.")
        .after_help(ARGS_AFTER_HELP)
        .arg(
            Arg::with_name("COMMAND")
                .help("The command to be run.")
                .long_help("The command to be run. It is relative to the pivoted root.")
                .required(true),
        )
        .arg(
            Arg::with_name("ARGS")
                .help("Arguments passed to command")
                .multiple(true),
        )
        .arg(
            Arg::with_name("new-root")
                .short("r")
                .long("new-root")
                .help("The new root of the sandbox")
                .takes_value(true)
                .long_help(
                    "The new root of the sandbox. The jail will pivot root\n\
                     to this folder prior to running the command.",
                ),
        )
        .arg(
            Arg::with_name("share-net")
                .long("share-net")
                .help("Whether to share the net namespace or not")
                .long_help(
                    "Whether to share the net namespae or not. Not sharing\n\
                     is more secure but it is also slow on multiple\n\
                     successive runs (Linux Kernel Bug).",
                ),
        )
        .arg(
            Arg::with_name("stdin")
                .long("stdin")
                .takes_value(true)
                .help("From where to redirect stdin")
                .long_help("From where to redirect stdin. The path must be outside the jail"),
        )
        .arg(
            Arg::with_name("stdout")
                .long("stdout")
                .takes_value(true)
                .help("Where to redirect stdout")
                .long_help("Where to redirect stdout. The path must be outside the jail"),
        )
        .arg(
            Arg::with_name("stderr")
                .long("stderr")
                .takes_value(true)
                .help("Where to redirect stderr")
                .long_help("Where to redirect stderr. The path must be outside the jail"),
        )
        .arg(
            Arg::with_name("wall-time")
                .long("wall-time")
                .short("wt")
                .takes_value(true)
                .help("Wall time limit")
                .long_help(
                    "Wall time limit. If the executable runs for more than this\n\
                     many seconds (in real time) it is killed.\n\
                     Given as an unsigned number followed by one of the following\n\
                     suffixes ns(nanoseconds), ms(milliseconds) or s(seconds)",
                ),
        )
        .arg(
            Arg::with_name("time")
                .long("time")
                .short("t")
                .takes_value(true)
                .help("User time limit")
                .long_help(
                    "User time limit. If the executable uses more user time than\n\
                     the amount given, it will be killed. Multiple threads running\n\
                     at the same time will add up their user time.\n\
                     Given as an unsigned number followed by one of the following\n\
                     suffixes: ns(nanoseconds), ms(milliseconds) or s(seconds)",
                ),
        )
        .arg(
            Arg::with_name("memory")
                .long("memory")
                .short("m")
                .takes_value(true)
                .help("Memory limit")
                .long_help(
                    "Memory limit. The maximum amount of memory (heap, data, swap) this\n\
                     program is allowed to use. Given as an unsigned number followed by\n\
                     one of usual suffixes b, kb, mb, gb, kib, mib, gib.",
                ),
        )
        .arg(
            Arg::with_name("instance-name")
                .long("instance-name")
                .short("i")
                .takes_value(true)
                .help("Instance name for cgroups")
                .long_help(
                    "Instance name for cgroups. If you plan on running multiple\n\
                     sandboxes at the same time, it is mandatory they be given\n\
                     different instance name, otherwise their user times will\n\
                     add up.",
                ),
        )
        .arg(
            Arg::with_name("cpuacct-controller")
                .long("cpuacct-controller")
                .takes_value(true)
                .help("cpuacct contrroller path")
                .long_help(
                    "cpuacct controller path. Must have write permissions with the\n\
                     user running the sandbox.",
                ),
        )
        .arg(
            Arg::with_name("memory-controller")
                .long("memory-controller")
                .takes_value(true)
                .help("memory contrroller path")
                .long_help(
                    "memory controller path. Must have write permissions with the\n\
                     user running the sandbox.",
                ),
        )
}
