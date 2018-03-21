extern crate ia_sandbox;
extern crate tempdir;

use std::fs::File;
use std::io::Write;
use std::time::Duration;

use ia_sandbox::config::{Mount, MountOptions, SpaceUsage};
use ia_sandbox::errors::{ChildError, Error, FFIError};

use tempdir::TempDir;

mod utils;
use utils::{LimitsBuilder, PivotRoot, RunInfoExt, TestRunnerHelper};
use utils::matchers::{CompareLimits, IsSuccess, KilledBySignal, MemoryLimitExceeded,
                      NonZeroExitStatus, TimeLimitExceeded, WallTimeLimitExceeded};

const HELLO_WORLD: &'static str = "./target/release/hello_world";

const EXIT_WITH_INPUT: &'static str = "./target/release/exit_with_input";

const EXIT_WITH_LAST_ARGUMENT: &'static str = "./target/release/exit_with_last_argument";

const KILL_WITH_SIGNAL_ARG: &'static str = "./target/debug/kill_with_signal_arg";

const SLEEP_2_SECOND: &'static str = "./target/release/sleep_2_seconds";

const LOOP_500_MS: &'static str = "./target/release/loop_500_ms";

const THREADS_LOOP_500_MS: &'static str = "./target/release/threads_loop_500_ms";

const ALLOCATE_20_MEGABYTES: &'static str = "./target/release/allocate_20_megabytes";

const THREADS_ALLOCATE_20_MEGABYTES: &'static str =
    "./target/release/threads_allocate_20_megabytes";

const EXIT_WITH_ARG_FILE: &'static str = "./target/release/exit_with_arg_file";

#[test]
fn test_basic_sandbox() {
    TestRunnerHelper::for_simple_exec("test_basic_sandbox", HELLO_WORLD, PivotRoot::DoNot)
        .config_builder()
        .build_and_run()
        .unwrap()
        .assert(IsSuccess)
}

#[test]
fn test_exec_failed() {
    match TestRunnerHelper::for_simple_exec("test_exec_failed", HELLO_WORLD, PivotRoot::DoNot)
        .config_builder()
        .command("missing")
        .build_and_run()
        .unwrap_err()
    {
        Error::ChildError(ChildError::FFIError(FFIError::ExecError { .. })) => (),
        err => assert!(false, "Expected exec error, got {}", err),
    }
}

#[test]
fn test_pivot_root() {
    TestRunnerHelper::for_simple_exec("test_pivot_root", HELLO_WORLD, PivotRoot::Pivot)
        .config_builder()
        .build_and_run()
        .unwrap()
        .assert(IsSuccess)
}

#[test]
fn test_unshare_net() {
    TestRunnerHelper::for_simple_exec("test_unshare_net", HELLO_WORLD, PivotRoot::Pivot)
        .config_builder()
        .share_net(false)
        .build_and_run()
        .unwrap()
        .assert(IsSuccess)
}

#[test]
fn test_redirect_stdin() {
    let mut helper =
        TestRunnerHelper::for_simple_exec("test_redirect_stdin", EXIT_WITH_INPUT, PivotRoot::Pivot);

    helper.write_file("input", b"0");
    let input_path = helper.file_path("input");
    helper
        .config_builder()
        .stdin(input_path)
        .build_and_run()
        .unwrap()
        .assert(IsSuccess);

    helper.write_file("input", b"23");
    helper
        .config_builder()
        .build_and_run()
        .unwrap()
        .assert(NonZeroExitStatus::new(23));
}

#[test]
fn test_redirect_stdout() {
    let mut helper =
        TestRunnerHelper::for_simple_exec("test_redirect_stdout", HELLO_WORLD, PivotRoot::Pivot);

    let output_path = helper.file_path("output");
    helper
        .config_builder()
        .stdout(&output_path)
        .build_and_run()
        .unwrap()
        .assert(IsSuccess);

    assert_eq!(helper.read_line(output_path), "Hello World!\n");
}

#[test]
fn test_redirect_stderr() {
    let mut helper =
        TestRunnerHelper::for_simple_exec("test_redirect_stderr", HELLO_WORLD, PivotRoot::Pivot);

    let stderr_path = helper.file_path("stderr");
    helper
        .config_builder()
        .stderr(&stderr_path)
        .build_and_run()
        .unwrap()
        .assert(IsSuccess);

    assert_eq!(helper.read_line(stderr_path), "Hello stderr!\n");
}

#[test]
fn test_arguments() {
    TestRunnerHelper::for_simple_exec("test_arguments", EXIT_WITH_LAST_ARGUMENT, PivotRoot::Pivot)
        .config_builder()
        .arg("0")
        .build_and_run()
        .unwrap()
        .assert(IsSuccess);

    TestRunnerHelper::for_simple_exec("test_arguments", EXIT_WITH_LAST_ARGUMENT, PivotRoot::Pivot)
        .config_builder()
        .args(vec!["24", "0", "17"])
        .build_and_run()
        .unwrap()
        .assert(NonZeroExitStatus::new(17))
}

#[test]
fn test_killed_by_signal() {
    TestRunnerHelper::for_simple_exec(
        "test_killed_by_signal",
        KILL_WITH_SIGNAL_ARG,
        PivotRoot::Pivot,
    ).config_builder()
        .arg("8")
        .build_and_run()
        .unwrap()
        .assert(KilledBySignal(8));

    TestRunnerHelper::for_simple_exec(
        "test_killed_by_signal",
        KILL_WITH_SIGNAL_ARG,
        PivotRoot::Pivot,
    ).config_builder()
        .arg("11")
        .build_and_run()
        .unwrap()
        .assert(KilledBySignal(11));
}

#[test]
fn test_wall_time_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits.wall_time(Duration::from_millis(2200));

    TestRunnerHelper::for_simple_exec(
        "test_wall_time_limit_exceeded",
        SLEEP_2_SECOND,
        PivotRoot::Pivot,
    ).config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(IsSuccess, limits));

    limits.wall_time(Duration::from_millis(1800));
    TestRunnerHelper::for_simple_exec(
        "test_wall_time_limit_exceeded",
        SLEEP_2_SECOND,
        PivotRoot::Pivot,
    ).config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(WallTimeLimitExceeded, limits));
}

#[test]
fn test_time_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits.user_time(Duration::from_millis(600));

    TestRunnerHelper::for_simple_exec("test_time_limit_exceeded", LOOP_500_MS, PivotRoot::Pivot)
        .config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(IsSuccess, limits));

    limits.user_time(Duration::from_millis(450));
    TestRunnerHelper::for_simple_exec("test_time_limit_exceeded", LOOP_500_MS, PivotRoot::Pivot)
        .config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(TimeLimitExceeded, limits));
}

#[test]
fn test_threads_time_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits.user_time(Duration::from_millis(600));

    TestRunnerHelper::for_simple_exec(
        "test_threads_time_limit_exceeded",
        THREADS_LOOP_500_MS,
        PivotRoot::Pivot,
    ).config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(IsSuccess, limits));

    limits.user_time(Duration::from_millis(450));
    TestRunnerHelper::for_simple_exec(
        "test_threads_time_limit_exceeded",
        THREADS_LOOP_500_MS,
        PivotRoot::Pivot,
    ).config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(TimeLimitExceeded, limits));
}

#[test]
fn test_threads_wall_time_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits
        .wall_time(Duration::from_millis(1000))
        .user_time(Duration::from_millis(600));

    TestRunnerHelper::for_simple_exec(
        "test_threads_wall_time_limit_exceeded",
        THREADS_LOOP_500_MS,
        PivotRoot::Pivot,
    ).config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(IsSuccess, limits));
}

#[test]
fn test_memory_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits.memory(SpaceUsage::from_megabytes(26));

    TestRunnerHelper::for_simple_exec(
        "test_memory_limit_exceeded",
        ALLOCATE_20_MEGABYTES,
        PivotRoot::Pivot,
    ).config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(IsSuccess, limits));

    limits.memory(SpaceUsage::from_megabytes(19));
    TestRunnerHelper::for_simple_exec(
        "test_memory_limit_exceeded",
        ALLOCATE_20_MEGABYTES,
        PivotRoot::Pivot,
    ).config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(MemoryLimitExceeded, limits));
}

#[test]
fn test_threads_memory_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits.memory(SpaceUsage::from_megabytes(40));

    TestRunnerHelper::for_simple_exec(
        "test_threads_memory_limit_exceeded",
        THREADS_ALLOCATE_20_MEGABYTES,
        PivotRoot::Pivot,
    ).config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(IsSuccess, limits));

    limits.memory(SpaceUsage::from_megabytes(19));
    TestRunnerHelper::for_simple_exec(
        "test_threads_memory_limit_exceeded",
        THREADS_ALLOCATE_20_MEGABYTES,
        PivotRoot::Pivot,
    ).config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(MemoryLimitExceeded, limits));
}

#[test]
fn test_pids_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits.pids(5);

    TestRunnerHelper::for_simple_exec(
        "test_pids_limit_exceeded",
        THREADS_ALLOCATE_20_MEGABYTES,
        PivotRoot::Pivot,
    ).config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(IsSuccess, limits));

    limits.pids(4);
    TestRunnerHelper::for_simple_exec(
        "test_pids_limit_exceeded",
        THREADS_ALLOCATE_20_MEGABYTES,
        PivotRoot::Pivot,
    ).config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(NonZeroExitStatus::any(), limits));
}

#[test]
fn test_mount_directory() {
    let temp_dir = TempDir::new("test_mount_directory_special").unwrap();
    let input_path = temp_dir.path().join("input");
    let mut file = File::create(&input_path).unwrap();
    let _ = file.write("15\n".as_bytes()).unwrap();

    TestRunnerHelper::for_simple_exec("test_mount_directory", EXIT_WITH_ARG_FILE, PivotRoot::Pivot)
        .config_builder()
        .mount(Mount::new(
            temp_dir.path().into(),
            "/mount".into(),
            MountOptions::default(),
        ))
        .arg("/mount/input")
        .build_and_run()
        .unwrap()
        .assert(NonZeroExitStatus::new(15));
}
