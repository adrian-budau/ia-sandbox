use clap::{App, AppSettings, Arg};

const ABOUT: &'static str = "
ia-jail sandboxes applications for secure running of executables";

const LONG_ABOUT: &'static str = "
ia-jail sandboxes applications for secure running of executables

ia-jail uses cgroups, namespaces, pivot_root and other techniques to
guarantee security. It is designed to be used for online judges and in
particular infoarena.ro";

const ARGS_AFTER_HELP: &'static str = "
All of the trailing arguments are passed to the command to run. If you're passing
arguments to both ia-jail and the binary, the ones after `--` go to the command,
the ones before go to ia-jail.";

pub fn app() -> App<'static, 'static> {
    App::new("ia-jail")
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
}
