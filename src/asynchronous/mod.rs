//! The asynchronous version of the borg commands are defined in this module

use std::io;
use std::path::Path;
use std::process::Output;

pub use compact::compact;
pub use create::{create, create_progress, CreateProgress};
pub use extract::extract;
pub use init::init;
pub use list::list;
pub use mount::{mount, umount};
pub use prune::prune;

mod compact;
mod create;
mod extract;
mod init;
mod list;
mod mount;
mod prune;

pub(crate) async fn execute_borg(
    local_path: &str,
    args: Vec<String>,
    passphrase: &Option<String>,
) -> Result<Output, io::Error> {
    execute_borg_with_current_dir(local_path, args, passphrase, None).await
}

pub(crate) async fn execute_borg_with_current_dir(
    local_path: &str,
    args: Vec<String>,
    passphrase: &Option<String>,
    current_dir: Option<&Path>,
) -> Result<Output, io::Error> {
    let mut command = tokio::process::Command::new(local_path);
    command.args(args);

    if let Some(passphrase) = passphrase {
        command.env("BORG_PASSPHRASE", passphrase);
    }

    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir);
    }

    command.output().await
}
