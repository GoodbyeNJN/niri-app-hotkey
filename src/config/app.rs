use knus::Decode;

use super::rule::WindowRule;

#[derive(Clone, Debug, Decode)]
pub struct App {
    #[knus(argument)]
    pub name: String,
    #[knus(child, unwrap(arguments))]
    pub spawn: Option<Vec<String>>,
    #[knus(child, unwrap(argument))]
    pub spawn_sh: Option<String>,
    #[knus(child)]
    pub window: Option<WindowRule>,
}
