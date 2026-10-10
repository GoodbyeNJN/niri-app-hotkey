use knus::{Decode, DecodeScalar};
use niri_ipc::Window;

use super::regex::Regex;

#[derive(Clone, Debug, Decode)]
pub struct RegexMatchRule {
    #[knus(property, str)]
    pub app_id: Option<Regex>,

    #[knus(property, str)]
    pub title: Option<Regex>,
}

impl RegexMatchRule {
    pub fn test_window(&self, window: &Window) -> bool {
        if let Some(app_id_re) = &self.app_id {
            let Some(app_id) = &window.app_id else {
                return false;
            };
            if !app_id_re.is_match(app_id) {
                return false;
            }
        }

        if let Some(title_re) = &self.title {
            let Some(title) = &window.title else {
                return false;
            };
            if !title_re.is_match(title) {
                return false;
            }
        }

        true
    }
}

#[derive(Clone, Debug, DecodeScalar)]
pub enum OrderByKey {
    Id,
    Title,
    AppId,
    Pid,
    WorkspaceId,
}

#[derive(Clone, Debug, Decode)]
pub struct OrderBy {
    #[knus(argument)]
    pub key: OrderByKey,
}

#[derive(Clone, Debug, Decode)]
pub struct SelectRule {
    #[knus(property)]
    pub index: Option<usize>,

    #[knus(children(name = "order-by"))]
    pub order_bys: Vec<OrderBy>,
}

impl SelectRule {
    pub fn select_window<'a>(&self, windows: &[&'a Window]) -> Option<&'a Window> {
        let Some(index) = self.index else {
            return None;
        };

        let mut ordered_windows = windows.to_vec();
        for order_by in self.order_bys.iter().rev() {
            match order_by.key {
                OrderByKey::Id => ordered_windows.sort_by_key(|w| &w.id),
                OrderByKey::AppId => ordered_windows.sort_by_key(|w| &w.app_id),
                OrderByKey::Title => ordered_windows.sort_by_key(|w| &w.title),
                OrderByKey::Pid => ordered_windows.sort_by_key(|w| &w.pid),
                OrderByKey::WorkspaceId => ordered_windows.sort_by_key(|w| &w.workspace_id),
            }
        }

        ordered_windows.get(index).copied()
    }
}

#[derive(Clone, Debug, Decode)]
pub struct WindowRule {
    #[knus(children(name = "match"))]
    pub matches: Vec<RegexMatchRule>,

    #[knus(children(name = "exclude"))]
    pub excludes: Vec<RegexMatchRule>,

    #[knus(child)]
    pub select: Option<SelectRule>,
}

impl WindowRule {
    pub fn test_window(&self, window: &Window) -> bool {
        if self.matches.is_empty() {
            return false;
        }

        let has_matches = &self.matches.iter().any(|m| m.test_window(window));
        if !has_matches {
            return false;
        }

        if self.excludes.is_empty() {
            return true;
        }

        let all_excludes = self.excludes.iter().all(|e| !e.test_window(window));
        if !all_excludes {
            return false;
        }

        true
    }

    pub fn select_window<'a>(&self, windows: &[&'a Window]) -> Option<&'a Window> {
        let Some(select) = &self.select else {
            return None;
        };

        select.select_window(windows)
    }
}
