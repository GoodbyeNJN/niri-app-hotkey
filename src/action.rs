use std::{
    cmp::Ordering,
    collections::HashSet,
    path::PathBuf,
    process::{Command, Stdio},
};

use directories::UserDirs;
use miette::{Context, IntoDiagnostic, Result, bail, miette};
use niri_ipc::{Action, Request, Response, Workspace, WorkspaceReferenceArg};
use niri_ipc::{Window, socket::Socket};

use crate::config::{Application, MatchRule};

fn expand_home(path: PathBuf) -> PathBuf {
    if let Ok(suffix) = path.strip_prefix("~") {
        if let Some(dirs) = UserDirs::new() {
            return dirs.home_dir().join(suffix);
        }
    }

    path
}

pub fn launch(application: &Application) -> Result<()> {
    let command: PathBuf;
    let args: Vec<String>;
    if let Some(spawn_command) = &application.spawn {
        let mut iter = spawn_command.iter();
        command = iter
            .next()
            .map(PathBuf::from)
            .map(expand_home)
            .ok_or_else(|| miette!("Spawn command is empty"))?;
        args = iter.cloned().collect();
    } else if let Some(spawn_sh_command) = &application.spawn_sh {
        command = PathBuf::from("sh");
        args = Vec::from(["-c".to_string(), spawn_sh_command.clone()]);
    } else {
        bail!(
            "No spawn command or spawn_sh command specified for application {}",
            application.name
        );
    };

    let mut process = Command::new(command);
    let process = process
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = process
        .spawn()
        .into_diagnostic()
        .context("Failed to spawn process")?;
    child
        .wait()
        .into_diagnostic()
        .context("Failed to wait for spawned process")?;

    Ok(())
}

fn is_window_match_rule(window: &Window, rule: &MatchRule) -> bool {
    if let Some(app_id_re) = &rule.app_id {
        let Some(app_id) = &window.app_id else {
            return false;
        };
        if !app_id_re.0.is_match(app_id) {
            return false;
        }
    }

    if let Some(title_re) = &rule.title {
        let Some(title) = &window.title else {
            return false;
        };
        if !title_re.0.is_match(title) {
            return false;
        }
    }

    true
}

fn match_windows_with_rules<'a>(
    windows: &'a Vec<Window>,
    rules: &'a Vec<MatchRule>,
) -> Vec<(Option<usize>, Vec<u64>)> {
    let mut mappings = vec![];
    for rule in rules {
        let mut matched_windows = windows
            .iter()
            .filter(|window| is_window_match_rule(window, rule))
            .collect::<Vec<_>>();
        matched_windows.sort_by(|a, b| match (a.pid, b.pid) {
            (Some(a_pid), Some(b_pid)) => a_pid.cmp(&b_pid),
            _ => Ordering::Equal,
        });

        mappings.push((
            rule.index,
            matched_windows.iter().map(|window| window.id).collect(),
        ));
    }

    mappings
}

fn get_matched_window_and_workspace<'a>(
    windows: &'a Vec<Window>,
    workspaces: &'a Vec<Workspace>,
    matches: &'a Vec<MatchRule>,
    excludes: &'a Vec<MatchRule>,
) -> Result<Option<(&'a Window, &'a Workspace)>> {
    // Collect excluded window IDs
    let mut excluded_window_ids = HashSet::new();
    for (rule_index, matched_window_ids) in match_windows_with_rules(windows, excludes) {
        for (index, window_id) in matched_window_ids.iter().enumerate() {
            match rule_index {
                Some(rule_index) if rule_index == index => {
                    excluded_window_ids.insert(*window_id);
                }
                None => {
                    excluded_window_ids.insert(*window_id);
                }
                _ => {}
            }
        }
    }

    // Collect included window IDs
    let mut include_window_ids = HashSet::new();
    for (rule_index, matched_window_ids) in match_windows_with_rules(&windows, matches) {
        for (index, window_id) in matched_window_ids.iter().enumerate() {
            match rule_index {
                Some(rule_index) if rule_index == index => {
                    include_window_ids.insert(*window_id);
                }
                None => {
                    include_window_ids.insert(*window_id);
                }
                _ => {}
            }
        }
    }

    // Final matched windows after applying excludes and includes
    let matched_windows = windows
        .iter()
        .filter(|window| {
            !excluded_window_ids.contains(&window.id) && include_window_ids.contains(&window.id)
        })
        .collect::<Vec<_>>();

    // Check results
    if matched_windows.len() > 1 {
        bail!(
            "Multiple windows matched the given rules. Refine your match/exclude rules to target a single window. Matched windows: {:#?}",
            matched_windows
        );
    }

    if matched_windows.is_empty() {
        return Ok(None);
    }

    // Get workspace of matched window
    let matched_window = matched_windows[0];
    let matched_window_workspace_id = matched_window.workspace_id.ok_or_else(|| {
        miette!(
            "Matched window with id {} does not belong to any workspace",
            matched_window.id
        )
    })?;
    let matched_window_workspace = workspaces
        .iter()
        .find(|workspace| workspace.id == matched_window_workspace_id)
        .ok_or_else(|| {
            miette!(
                "Workspace with id {} not found for matched window",
                matched_window_workspace_id
            )
        })?;

    Ok(Some((matched_window, matched_window_workspace)))
}

fn get_focused_window(windows: &Vec<Window>) -> Option<&Window> {
    windows.iter().find(|window| window.is_focused)
}

fn get_focused_workspace(workspaces: &Vec<Workspace>) -> Result<&Workspace> {
    workspaces
        .iter()
        .find(|workspace| workspace.is_focused)
        .ok_or_else(|| miette!("No focused workspace found"))
}

fn get_hidden_workspace(workspaces: &Vec<Workspace>) -> Result<&Workspace> {
    workspaces
        .iter()
        .find(|workspace| workspace.is_hidden)
        .ok_or_else(|| miette!("No hidden workspace found"))
}

fn get_window_and_workspace_list(socket: &mut Socket) -> Result<(Vec<Window>, Vec<Workspace>)> {
    let (Ok(Response::Windows(windows)), Ok(Response::Workspaces(workspaces))) = (
        socket.send(Request::Windows).into_diagnostic()?,
        socket
            .send(Request::WorkspacesWithHidden)
            .into_diagnostic()?,
    ) else {
        bail!("Failed to retrieve windows or workspaces from Niri daemon");
    };

    Ok((windows, workspaces))
}

pub fn show(application: &Application) -> Result<()> {
    let mut socket = Socket::connect().into_diagnostic()?;
    let (windows, workspaces) = get_window_and_workspace_list(&mut socket)?;

    let (matched_window, matched_window_workspace) = get_matched_window_and_workspace(
        &windows,
        &workspaces,
        &application.matches,
        &application.excludes,
    )?
    .ok_or_else(|| miette!("No window matched the given rules."))?;

    let focused_workspace = get_focused_workspace(&workspaces)?;
    if focused_workspace.id != matched_window_workspace.id {
        // Move the matched window to focused workspace and focus it
        let _ = socket
            .send(Request::Action(Action::MoveWindowToWorkspace {
                window_id: Some(matched_window.id),
                reference: WorkspaceReferenceArg::Id(focused_workspace.id),
                focus: true,
            }))
            .into_diagnostic()?;
    };

    // Matched window is already in focused workspace, just focus it
    let _ = socket
        .send(Request::Action(Action::FocusWindow {
            id: matched_window.id,
        }))
        .into_diagnostic()?;

    Ok(())
}

pub fn hide(application: &Application) -> Result<()> {
    let mut socket = Socket::connect().into_diagnostic()?;
    let (windows, workspaces) = get_window_and_workspace_list(&mut socket)?;

    let (matched_window, matched_window_workspace) = get_matched_window_and_workspace(
        &windows,
        &workspaces,
        &application.matches,
        &application.excludes,
    )?
    .ok_or_else(|| miette!("No window matched the given rules."))?;

    let focused_window =
        get_focused_window(&windows).ok_or_else(|| miette!("No focused window found"))?;
    if focused_window.id != matched_window.id {
        bail!("The matched window is not focused, cannot hide it.");
    }

    let hidden_workspace = get_hidden_workspace(&workspaces)?;
    if hidden_workspace.id == matched_window_workspace.id {
        bail!("The matched window is already in the hidden workspace.");
    }

    // Move focused window to hidden workspace
    let _ = socket
        .send(Request::Action(Action::MoveWindowToWorkspace {
            window_id: Some(matched_window.id),
            reference: WorkspaceReferenceArg::Id(hidden_workspace.id),
            focus: false,
        }))
        .into_diagnostic()?;

    Ok(())
}

pub fn activate(application: &Application) -> Result<()> {
    let mut socket = Socket::connect().into_diagnostic()?;
    let (windows, workspaces) = get_window_and_workspace_list(&mut socket)?;

    let (matched_window, matched_window_workspace) = get_matched_window_and_workspace(
        &windows,
        &workspaces,
        &application.matches,
        &application.excludes,
    )?
    .ok_or_else(|| miette!("No window matched the given rules."))?;

    let focused_workspace = get_focused_workspace(&workspaces)?;
    if focused_workspace.id != matched_window_workspace.id {
        bail!("The matched window is not in the focused workspace, cannot activate it.");
    }

    // Focus the matched window
    let _ = socket
        .send(Request::Action(Action::FocusWindow {
            id: matched_window.id,
        }))
        .into_diagnostic()?;

    Ok(())
}

pub fn toggle(application: &Application) -> Result<()> {
    let mut socket = Socket::connect().into_diagnostic()?;
    let (windows, workspaces) = get_window_and_workspace_list(&mut socket)?;

    let matched = get_matched_window_and_workspace(
        &windows,
        &workspaces,
        &application.matches,
        &application.excludes,
    )?;
    if matched.is_none() {
        // No matched window, launch the application
        return launch(&application);
    }

    let (matched_window, matched_window_workspace) = matched.unwrap();

    if let Some(focused_window) = get_focused_window(&windows) {
        if focused_window.id == matched_window.id {
            // Matched window is focused, hide it
            let hidden_workspace = get_hidden_workspace(&workspaces)?;
            let _ = socket
                .send(Request::Action(Action::MoveWindowToWorkspace {
                    window_id: Some(matched_window.id),
                    reference: WorkspaceReferenceArg::Id(hidden_workspace.id),
                    focus: false,
                }))
                .into_diagnostic()?;
            return Ok(());
        }
    }

    let focused_workspace = get_focused_workspace(&workspaces)?;
    if focused_workspace.id != matched_window_workspace.id {
        // Move matched window to focused workspace and focus it
        let _ = socket
            .send(Request::Action(Action::MoveWindowToWorkspace {
                window_id: Some(matched_window.id),
                reference: WorkspaceReferenceArg::Id(focused_workspace.id),
                focus: true,
            }))
            .into_diagnostic()?;
    }

    // Matched window is in focused workspace, focus it
    let _ = socket
        .send(Request::Action(Action::FocusWindow {
            id: matched_window.id,
        }))
        .into_diagnostic()?;

    Ok(())
}
