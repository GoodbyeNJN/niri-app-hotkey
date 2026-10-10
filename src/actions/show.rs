use anyhow::Result;

use super::helpers::{should_match_window_and_workspace, try_get_matched_window_and_workspace};
use crate::{App, Niri};

pub fn show(app: &App) -> Result<()> {
    if !should_match_window_and_workspace(app) {
        return Ok(());
    }

    let mut niri = Niri::connect()?;
    let windows = niri.get_windows()?;
    let workspaces = niri.get_workspaces()?;

    let (matched_window, matched_workspace) =
        try_get_matched_window_and_workspace(app, &windows, &workspaces)?;

    let focused_workspace = Niri::try_get_focused_workspace(&workspaces)?;
    if focused_workspace.id != matched_workspace.id {
        // Move the matched window to focused workspace and focus it
        niri.move_window_to_workspace(matched_window, focused_workspace, true)?;
    };

    // Matched window is already in focused workspace, just focus it
    niri.focus_window(matched_window)
}
