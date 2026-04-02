use log::{debug, info};

use crate::asynchronous::execute_borg_with_current_dir;
use crate::common::{extract_fmt_args, extract_parse_output, CommonOptions, ExtractOptions};
use crate::errors::ExtractError;

/// Extract the contents of an archive.
///
/// This command extracts the contents of an archive to the specified destination.
/// By default, the entire archive is extracted but you can use the strip_components
/// option to remove leading path elements.
///
/// **Parameter**:
/// - `options`: Reference to [ExtractOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
pub async fn extract(
    options: &ExtractOptions,
    common_options: &CommonOptions,
) -> Result<(), ExtractError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    let args = extract_fmt_args(options, common_options);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(ExtractError::ShlexError)?;
    let res = execute_borg_with_current_dir(
        local_path,
        args,
        &options.passphrase,
        Some(std::path::Path::new(&options.destination)),
    )
    .await?;
    extract_parse_output(res)?;

    info!("Finished extracting archive");

    Ok(())
}
