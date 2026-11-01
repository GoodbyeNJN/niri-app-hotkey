use anyhow::{Ok, Result, anyhow, bail};
use niri_ipc::Workspace;

use crate::{App, Window};

pub(super) fn should_match_window_and_workspace(app: &App) -> bool {
    app.window.is_some()
}

pub(super) fn try_get_matched_window_and_workspace<'a>(
    app: &App,
    windows: &'a Vec<Window>,
    workspaces: &'a Vec<Workspace>,
) -> Result<(&'a Window, &'a Workspace)> {
    match get_matched_window_and_workspace(app, windows, workspaces)? {
        Some((window, workspace)) => Ok((window, workspace)),
        None => bail!("No window matched the window rules"),
    }
}

pub(super) fn get_matched_window_and_workspace<'a>(
    app: &App,
    windows: &'a Vec<Window>,
    workspaces: &'a Vec<Workspace>,
) -> Result<Option<(&'a Window, &'a Workspace)>> {
    let matched_window = {
        let Some(window_rule) = &app.window else {
            return Ok(None);
        };

        let matched_windows = windows
            .iter()
            .filter(|window| window_rule.test_window(window))
            .collect::<Vec<_>>();

        match matched_windows.len() {
            0 => return Ok(None),
            1 => matched_windows[0],
            _ => match window_rule.select_window(&matched_windows) {
                Some(window) => window,
                None => {
                    return Err(anyhow!(
                        "Multiple windows matched the window rules, but none could be selected as the target window"
                    ).context(format!("Matched window list: {matched_windows:#?}")));
                }
            },
        }
    };

    let matched_workspace = {
        let Some(workspace_id) = matched_window.workspace_id else {
            bail!(
                "Matched window with id '{}' does not belong to any workspace",
                matched_window.id
            );
        };

        let matched_workspace = workspaces
            .iter()
            .find(|workspace| workspace.id == workspace_id);

        match matched_workspace {
            Some(workspace) => workspace,
            None => bail!("Workspace with id '{workspace_id}' not found for matched window"),
        }
    };

    Ok(Some((matched_window, matched_workspace)))
}
