use crate::runners::BackgroundRunner;
use crate::states::GlobalStates;
use std::sync::Arc;

mod assets;
mod runners;
mod states;

#[derive(Clone)]
pub struct AppContext {
    inner: Arc<AppContextInner>,
}

struct AppContextInner {
    global_states: GlobalStates,
    background_runner: BackgroundRunner,
}
