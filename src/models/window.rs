use std::fs;
use std::ops::Deref;

use niri_ipc::Window as NiriWindow;

#[derive(Debug, Clone)]
pub struct Window {
    niri_window: NiriWindow,
    pub program: Option<String>,
    pub cmdline: Option<String>,
}

impl From<NiriWindow> for Window {
    fn from(niri_window: NiriWindow) -> Self {
        let program = niri_window.pid.and_then(|pid| {
            fs::read_link(format!("/proc/{pid}/exe"))
                .map(|path| path.to_string_lossy().to_string())
                .ok()
        });
        let cmdline = niri_window.pid.and_then(|pid| {
            fs::read(format!("/proc/{pid}/cmdline"))
                .map(|bytes| {
                    bytes
                        .split(|&b| b == 0) // split by null byte
                        .filter(|slice| !slice.is_empty())
                        .map(|slice| String::from_utf8_lossy(slice).to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .ok()
        });

        Self {
            niri_window,
            program,
            cmdline,
        }
    }
}

impl From<&NiriWindow> for Window {
    fn from(niri_window: &NiriWindow) -> Self {
        Self::from(niri_window.clone())
    }
}

impl Deref for Window {
    type Target = NiriWindow;

    fn deref(&self) -> &Self::Target {
        &self.niri_window
    }
}
