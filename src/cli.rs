use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, name = "bump2version", propagate_version = true)]
pub(crate) struct Cli {
    /// Config file to read most of the variables from.
    #[arg(
        short = 'c',
        long = "config-file",
        value_name = "FILE",
        default_value_t = String::from(".bumpversion.toml")
    )]
    pub(crate) config_file: String,

    /// Part of the version to be bumped.
    #[arg(
        long = "bump",
        value_name = "PART",
        default_value_t = String::from("patch")
    )]
    pub(crate) bump: String,

    /// Don't write any files, just pretend.
    #[arg(short = 'n', long = "dry-run", default_value_t = false)]
    pub(crate) dry_run: bool,

    /// New version that should be in the files.
    #[arg(long = "new-version", value_name = "VERSION")]
    pub(crate) new_version: Option<String>,

    /// Create a commit in version control.
    #[arg(long = "commit")]
    pub(crate) commit: Option<bool>,

    /// Create a tag in version control.
    #[arg(long = "tag")]
    pub(crate) tag: Option<bool>,

    /// Whether to fail on a dirty git repository
    #[arg(long = "fail-on-dirty", default_value_t = false)]
    pub(crate) fail_on_dirty: bool,

    /// Commit message.
    #[arg(short = 'm', long = "message", value_name = "COMMIT_MSG")]
    pub(crate) message: Option<String>,
}
