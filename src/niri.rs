use anyhow::{Error, Result, bail};
use niri_ipc::socket::Socket;
use niri_ipc::{Action, Request, Response, Window, Workspace, WorkspaceReferenceArg};

pub struct Niri {
    socket: Socket,
}

impl Niri {
    pub fn connect() -> Result<Self> {
        let socket = Socket::connect()?;

        Ok(Self { socket })
    }

    pub fn get_windows(&mut self) -> Result<Vec<Window>> {
        let windows = self.socket.send(Request::Windows)?.map_err(Error::msg)?;

        let Response::Windows(mut windows) = windows else {
            bail!("Failed to retrieve windows from Niri daemon")
        };
        windows.sort_by_key(|window| window.id);

        Ok(windows)
    }

    pub fn get_workspaces(&mut self) -> Result<Vec<Workspace>> {
        let workspaces = self
            .socket
            .send(Request::WorkspacesWithHidden)?
            .map_err(Error::msg)?;

        let Response::Workspaces(mut workspaces) = workspaces else {
            bail!("Failed to retrieve workspaces from Niri daemon")
        };
        workspaces.sort_by_key(|workspace| workspace.id);

        Ok(workspaces)
    }

    pub fn get_focused_window(windows: &Vec<Window>) -> Option<&Window> {
        windows.iter().find(|window| window.is_focused)
    }

    pub fn try_get_focused_window(windows: &Vec<Window>) -> Result<&Window> {
        match Self::get_focused_window(windows) {
            Some(window) => Ok(window),
            None => bail!("No focused window found"),
        }
    }

    pub fn get_focused_workspace(workspaces: &Vec<Workspace>) -> Option<&Workspace> {
        workspaces.iter().find(|workspace| workspace.is_focused)
    }

    pub fn try_get_focused_workspace(workspaces: &Vec<Workspace>) -> Result<&Workspace> {
        match Self::get_focused_workspace(workspaces) {
            Some(workspace) => Ok(workspace),
            None => bail!("No focused workspace found"),
        }
    }

    pub fn get_hidden_workspace(workspaces: &Vec<Workspace>) -> Option<&Workspace> {
        workspaces.iter().find(|workspace| workspace.is_hidden)
    }

    pub fn try_get_hidden_workspace(workspaces: &Vec<Workspace>) -> Result<&Workspace> {
        match Self::get_hidden_workspace(workspaces) {
            Some(workspace) => Ok(workspace),
            None => bail!("No hidden workspace found"),
        }
    }

    pub fn focus_window(&mut self, window: &Window) -> Result<()> {
        let response = self
            .socket
            .send(Request::Action(Action::FocusWindow { id: window.id }))?
            .map_err(Error::msg)?;

        let Response::Handled = response else {
            bail!("Failed to focus window")
        };

        Ok(())
    }

    pub fn move_window_to_workspace(
        &mut self,
        window: &Window,
        workspace: &Workspace,
        focus: bool,
    ) -> Result<()> {
        let response = self
            .socket
            .send(Request::Action(Action::MoveWindowToWorkspace {
                window_id: Some(window.id),
                reference: WorkspaceReferenceArg::Id(workspace.id),
                focus,
            }))?
            .map_err(Error::msg)?;

        let Response::Handled = response else {
            bail!("Failed to move window to workspace")
        };

        Ok(())
    }
}
