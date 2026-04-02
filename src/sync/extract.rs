use log::{debug, info};

use crate::common::{extract_fmt_args, extract_parse_output, CommonOptions, ExtractOptions};
use crate::errors::ExtractError;
use crate::sync::execute_borg_with_current_dir;

/// Extract the contents of an archive.
///
/// This command extracts the contents of an archive to the specified destination.
/// By default, the entire archive is extracted but you can use the strip_components
/// option to remove leading path elements.
///
/// **Parameter**:
/// - `options`: Reference to [ExtractOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
pub fn extract(
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
    )?;
    extract_parse_output(res)?;

    info!("Finished extracting archive");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::thread;
    use std::time::{Duration, Instant};

    use crate::common::{CommonOptions, ExtractOptions};
    use crate::sync::extract;

    #[cfg(unix)]
    fn create_fake_borg(path: &std::path::Path) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let script = path.join("fake-borg.sh");
        fs::write(&script, "#!/bin/sh\nsleep 0.2\npwd > pwd.txt\n").unwrap();

        let mut permissions = fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script, permissions).unwrap();

        script
    }

    #[cfg(unix)]
    #[test]
    fn concurrent_extracts_do_not_change_process_cwd() {
        let temp = tempfile::tempdir().unwrap();
        let dest1 = temp.path().join("dest-1");
        let dest2 = temp.path().join("dest-2");
        fs::create_dir(&dest1).unwrap();
        fs::create_dir(&dest2).unwrap();

        let borg = create_fake_borg(temp.path());
        let common = CommonOptions {
            local_path: Some(borg.to_string_lossy().into_owned()),
            ..CommonOptions::default()
        };

        let options1 = ExtractOptions::new(
            "repo".to_string(),
            "archive-1".to_string(),
            dest1.to_string_lossy().into_owned(),
        );
        let options2 = ExtractOptions::new(
            "repo".to_string(),
            "archive-2".to_string(),
            dest2.to_string_lossy().into_owned(),
        );

        let original_dir = env::current_dir().unwrap();

        let common1 = common.clone();
        let t1 = thread::spawn(move || extract(&options1, &common1));

        let wait_deadline = Instant::now() + Duration::from_millis(150);
        while Instant::now() < wait_deadline {
            if env::current_dir().unwrap() != original_dir {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }

        let common2 = common.clone();
        let t2 = thread::spawn(move || extract(&options2, &common2));

        t1.join().unwrap().unwrap();
        t2.join().unwrap().unwrap();

        assert_eq!(env::current_dir().unwrap(), original_dir);
        assert_eq!(
            fs::read_to_string(dest1.join("pwd.txt")).unwrap().trim(),
            dest1.to_string_lossy()
        );
        assert_eq!(
            fs::read_to_string(dest2.join("pwd.txt")).unwrap().trim(),
            dest2.to_string_lossy()
        );
    }
}
