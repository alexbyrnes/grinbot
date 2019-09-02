#[derive(Default, Clone)]
pub struct State {
    pub screen: Screen, 
    pub prev_screen: Screen,
    pub id: Option<i64>,
}

#[derive(Debug, Clone)]
pub enum Screen {
    Home,
    Create,
    Send,
    Help
}

impl Default for Screen {
    fn default() -> Screen {
        Screen::Home
    }
}

#[derive(Debug)]
pub enum Action {
    Home(i64),
    Create(i64),
    Send(i64),
    Help(i64),
    Back(i64),
    Unknown(i64)
}

pub fn screen_reducer(state: &State, action: &Action) -> State {
    let s = state.clone();
    match *action {
        Action::Home(id) => State {
            prev_screen: Screen::Home,
            screen: Screen::Home, 
            id: Some(id),
        },
        Action::Create(id) => State {
            prev_screen: s.screen,
            screen: Screen::Create, 
            id: Some(id),
        },
        Action::Send(id) => State {
            prev_screen: s.screen,
            screen: Screen::Send, 
            id: Some(id),
        },
        Action::Help(id) => State {
            prev_screen: s.screen,
            screen: Screen::Help, 
            id: Some(id),
        },
        Action::Back(id) => State {
            prev_screen: Screen::Home,
            screen: s.prev_screen,  
            id: Some(id),
        },
        Action::Unknown(id) => State {
            prev_screen: s.prev_screen,
            screen: s.screen,  
            id: Some(id),
        }
    }
}

