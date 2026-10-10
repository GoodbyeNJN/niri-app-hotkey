use anyhow::Result;

use super::helpers::{get_matched_window_and_workspace, should_match_window_and_workspace};
use super::launch::launch;
use crate::{App, Niri};

pub fn toggle(app: &App) -> Result<()> {
    if !should_match_window_and_workspace(app) {
        // No need to match a window, launch the application
        return launch(app);
    }

    let mut niri = Niri::connect()?;
    let windows = niri.get_windows()?;
    let workspaces = niri.get_workspaces()?;

    let Some((matched_window, matched_window_workspace)) =
        get_matched_window_and_workspace(app, &windows, &workspaces)?
    else {
        // No matched window found, launch the application
        return launch(app);
    };

    let focused_window = Niri::get_focused_window(&windows);
    if let Some(focused_window) = focused_window
        && focused_window.id == matched_window.id
    {
        // Matched window is focused, hide it
        let hidden_workspace = Niri::try_get_hidden_workspace(&workspaces)?;
        niri.move_window_to_workspace(matched_window, hidden_workspace, false)?;

        return Ok(());
    }

    let focused_workspace = Niri::try_get_focused_workspace(&workspaces)?;
    if focused_workspace.id != matched_window_workspace.id {
        // Move matched window to focused workspace and focus it
        niri.move_window_to_workspace(matched_window, focused_workspace, true)?;
    }

    // Matched window is in focused workspace, focus it
    niri.focus_window(matched_window)
}
