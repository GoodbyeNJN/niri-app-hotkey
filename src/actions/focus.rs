use anyhow::{Result, bail};

use super::helpers::{should_match_window_and_workspace, try_get_matched_window_and_workspace};
use crate::{App, Niri};

pub fn focus(app: &App) -> Result<()> {
    if !should_match_window_and_workspace(app) {
        return Ok(());
    }

    let mut niri = Niri::connect()?;
    let windows = niri.get_windows()?;
    let workspaces = niri.get_workspaces()?;

    let (matched_window, matched_window_workspace) =
        try_get_matched_window_and_workspace(app, &windows, &workspaces)?;

    let focused_workspace = Niri::try_get_focused_workspace(&workspaces)?;
    if focused_workspace.id != matched_window_workspace.id {
        bail!("The matched window is not in the focused workspace, cannot activate it");
    }

    // Focus the matched window
    niri.focus_window(matched_window)
}
