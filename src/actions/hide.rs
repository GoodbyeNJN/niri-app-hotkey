use anyhow::{Result, bail};

use super::helpers::{should_match_window_and_workspace, try_get_matched_window_and_workspace};
use crate::{App, Niri};

pub fn hide(app: &App) -> Result<()> {
    if !should_match_window_and_workspace(app) {
        return Ok(());
    }

    let mut niri = Niri::connect()?;
    let windows = niri.get_windows()?;
    let workspaces = niri.get_workspaces()?;

    let (matched_window, matched_workspace) =
        try_get_matched_window_and_workspace(app, &windows, &workspaces)?;

    let focused_window = Niri::try_get_focused_window(&windows)?;
    if focused_window.id != matched_window.id {
        bail!("The matched window is not focused, cannot hide it");
    }

    let hidden_workspace = Niri::try_get_hidden_workspace(&workspaces)?;
    if hidden_workspace.id == matched_workspace.id {
        bail!("The matched window is already in the hidden workspace");
    }

    // Move focused window to hidden workspace
    niri.move_window_to_workspace(matched_window, hidden_workspace, false)
}
