use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use directories::UserDirs;

use crate::config::App;

fn expand_home(path: PathBuf) -> PathBuf {
    let Ok(suffix) = path.strip_prefix("~") else {
        return path;
    };
    let Some(dirs) = UserDirs::new() else {
        return path;
    };

    dirs.home_dir().join(suffix)
}

pub fn launch(app: &App) -> Result<()> {
    let (command, args) = match (&app.spawn, &app.spawn_sh) {
        (Some(spawn), _) => {
            let mut iter = spawn.iter();
            let Some(command) = iter.next().map(PathBuf::from).map(expand_home) else {
                bail!("Spawn command is empty");
            };
            let args = iter.cloned().collect();

            (command, args)
        }

        (None, Some(spawn_sh_command)) => {
            let command = PathBuf::from("sh");
            let args = Vec::from(["-c".into(), spawn_sh_command.into()]);

            (command, args)
        }

        _ => bail!(
            "No spawn command or spawn_sh command specified for app {}",
            app.name
        ),
    };

    let mut process = Command::new(command);
    let process = process
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let _ = process.spawn().context("Failed to spawn process")?;
    // child
    //     .wait()
    //     .into_diagnostic()
    //     .context("Failed to wait for spawned process")?;

    Ok(())
}
