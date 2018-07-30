use clap::{App, AppSettings, Arg};

const ABOUT: &str = "
ia-sandbox sandboxes applications for secure running of executables";

const LONG_ABOUT: &str = "
ia-sandbox sandboxes applications for secure running of executables

ia-sandbox uses cgroups, namespaces, pivot_root and other techniques to
guarantee security. It is designed to be used for online judges and in
particular infoarena.ro";

const ARGS_AFTER_HELP: &str = "
All of the trailing arguments are passed to the command to run. If you're passing
arguments to both ia-sandbox and the binary, the ones after `--` go to the command,
the ones before go to ia-sandbox.";

pub(crate) fn app() -> App<'static, 'static> {
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
                    "Whether to share the net namespace or not. Not sharing\n\
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
                     one of the usual suffixes b, kb, mb, gb, kib, mib, gib.",
                ),
        )
        .arg(
            Arg::with_name("stack")
                .long("stack")
                .short("s")
                .takes_value(true)
                .help("Stack memory limit")
                .long_help(
                    "Stack memory limit. The maxim amount of memory this program is allowed\n\
                     tu use as stack. Given as an unsigned number followed by\n\
                     one of the usual suffixes b, kb, mb, gb, kib, mib, gib.",
                ),
        )
        .arg(
            Arg::with_name("pids")
                .long("pids")
                .short("p")
                .takes_value(true)
                .default_value("50")
                .help("Number of pids limit")
                .long_help(
                    "Number of pids limit. The maximum amount of tasks (processes / threads)\n\
                     this program is allowed to create (count includes the program itself).\n\
                     Defaults 50 to protect against fork bombs.",
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
        .arg(
            Arg::with_name("pids-controller")
                .long("pids-controller")
                .takes_value(true)
                .help("pids controller path")
                .long_help(
                    "pids controller path. Must have write permissions with then\n\
                     user running the sandbox.",
                ),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .possible_values(&["human", "oneline", "json"])
                .default_value("human")
                .help("how to output the run information.")
                .long_help(
                    "how to output the run information.\n\
                     human - multiline string describing everything, not suitable for\n\
                     parsing.\n\
                     oneline - USER_TIME MEMORY VERDICT\n\
                     json - a single json object with 4 fields\n",
                ),
        )
        .arg(
            Arg::with_name("mount")
                .long("mount")
                .multiple(true)
                .number_of_values(1)
                .requires("new-root")
                .help("which files/folders to mount inside the new root")
                .long_help(
                    "which files/folders to mount inside the new root.\n\
                     Given in any of the following 3 forms:\n\
                     - source:destination:mount_options\n\
                     - source:destination (equivalent to source:destination:ro,noexec,)\n\
                     - source (equivalent to source:source)\n\
                     Mount options are given as a comma separated list of the following:\n\
                     - rw, default is to mount read-only\n\
                     - exec, default is to mount with no exec permissions\n\
                     - dev, default is to mount with no access to devices\n",
                ),
        )
        .arg(
            Arg::with_name("swap-redirects")
                .long("swap-redirects")
                .requires("stdin")
                .requires("stdout")
                .help("whether to reverse the opening of stdin and stdout")
                .long_help(
                    "whether to reverse the opening of stdin and stdout.\n\
                     When opening a FIFO filo for reading/writing, if it's not\n\
                     opened for writing/reading by another process then the current one\n\
                     is blocked. For 2 processes to communicate using 2 FIFO files\n\
                     one must open the input and then the output, and the other one must\n\
                     open output and then input.",
                ),
        )
        .arg(
            Arg::with_name("no-clear-usage")
                .long("no-clear-usage")
                .help("whether to not clear usage (time/memory/pids) from cgroups")
                .conflicts_with("time")
                .conflicts_with("memory")
                .conflicts_with("pids")
                .long_help(
                    "whether to not clear usage (time/memory/pids) from cgroups.\n\
                     For multi-run tasks cpu usage might be added for all run of the task.\n\
                     Because usage is not cleared, it does not make sense to change limits\n\
                     so this option conflicts with time/memory/pids limits.",
                ),
        )
        .arg(
            Arg::with_name("interactive")
                .long("interactive")
                .help("whether to run in interactive mode.")
                .conflicts_with("stdin")
                .long_help(
                    "whether to run in interactive mode. This is necessary if you would\n\
                     rather supply the standard input (instead of redirecting it from a\n\
                     file), like for example to run a bash shell.",
                ),
        )
        .arg(
            Arg::with_name("env")
                .long("env")
                .short("e")
                .multiple(true)
                .number_of_values(1)
                .help("an environment variable to pass to the process inside the sandbox")
                .long_help(
                    "an environment variable to pass to the process inside the sandbox.\n\
                     Given as NAME=VALUE.",
                ),
        )
        .arg(
            Arg::with_name("forward-env")
                .long("forward-env")
                .help("whether to forward all environment variables")
                .conflicts_with("env")
                .long_help(
                    "whether to forward all environmnet variabiles. If starting a shell\n\
                     this is useful for setting up proper functionality. Be careful as \n\
                     this might expose sensitive information.",
                ),
        )
}
