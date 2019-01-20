//! A simple example demonstrating how to implement a Todo app using Reducer & libui.

extern crate iui;
extern crate reducer;

use iui::controls::*;
use iui::prelude::*;
use reducer::*;
use std::error::Error;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum View {
    All,
    Done,
    Pending,
}

impl Default for View {
    fn default() -> Self {
        View::All
    }
}

impl View {
    fn name(self) -> &'static str {
        match self {
            View::All => "All",
            View::Done => "Done",
            View::Pending => "Pending",
        }
    }
}

// The actions a user can trigger on our app.
#[derive(Debug, Clone)]
enum Action {
    AddTodo(String),
    ToggleTodo(usize),
    FilterTodos(View),
}

// The state of our app.
#[derive(Debug, Default, Clone)]
struct State {
    todos: Vec<(bool, String)>,
    filter: View,
}

impl Reducer<Action> for State {
    // The entire business logic of our app goes here.
    fn reduce(&mut self, action: Action) {
        match action {
            Action::AddTodo(todo) => {
                if !todo.is_empty() {
                    self.todos.push((false, todo));
                }
            }

            Action::ToggleTodo(i) => {
                let (done, _) = &mut self.todos[i];
                *done = !*done;
            }

            Action::FilterTodos(filter) => {
                self.filter = filter;
            }
        }
    }
}

impl State {
    // These associated functions project the state into derived
    // properties that are more convenient for rendering.

    fn get_filter(&self) -> View {
        self.filter
    }

    fn get_todos(&self) -> &[(bool, String)] {
        &self.todos
    }
}

fn run_libui(dispatcher: impl Dispatcher<Action> + Clone + 'static, states: Receiver<Arc<State>>) {
    let ui = UI::init().unwrap();

    // Layout.
    let mut body = VerticalBox::new(&ui);
    body.set_padded(&ui, true);

    let mut header = HorizontalBox::new(&ui);
    header.set_padded(&ui, true);

    // Text input.
    let input = Entry::new(&ui);

    // "Add Todo" button.
    let mut add = Button::new(&ui, "Add Todo");
    add.on_clicked(&ui, {
        let mut input = input.clone();
        let mut dispatcher = dispatcher.clone();
        let ui = ui.clone();

        move |_| {
            dispatcher.dispatch(Action::AddTodo(input.value(&ui)));
            input.set_value(&ui, "");
        }
    });

    // View control.
    const VIEWS: [View; 3] = [View::All, View::Done, View::Pending];
    let mut filter = Combobox::new(&ui);
    filter.on_selected(&ui, {
        let mut dispatcher = dispatcher.clone();
        move |i| {
            dispatcher.dispatch(Action::FilterTodos(VIEWS[i as usize]));
        }
    });

    for view in &VIEWS {
        filter.append(&ui, view.name());
    }

    filter.set_selected(&ui, 0);

    let mut event_loop = ui.event_loop();

    event_loop.on_tick(&ui, {
        // keep track of todos displayed as libui doesn't yet provide a way of introspecting that.
        let mut checkboxes: Vec<Checkbox> = vec![];
        let mut body = body.clone();
        let mut filter = filter.clone();
        let dispatcher = dispatcher.clone();
        let ui = ui.clone();

        move || {
            // Update widgets on state change.
            if let Some(state) = states.try_iter().last() {
                // Add new todos
                let todos = state.get_todos();
                for (i, (_, todo)) in todos.iter().enumerate().skip(checkboxes.len()) {
                    let mut checkbox = Checkbox::new(&ui, todo);
                    checkbox.on_toggled(&ui, {
                        let mut dispatcher = dispatcher.clone();
                        move |_| {
                            dispatcher.dispatch(Action::ToggleTodo(i));
                        }
                    });
                    checkboxes.push(checkbox.clone());
                    body.append(&ui, checkbox, LayoutStrategy::Compact);
                }

                // Synchronize checkboxes with the state.
                for (&(done, _), checkbox) in todos.iter().zip(checkboxes.iter_mut()) {
                    checkbox.set_checked(&ui, done);
                    match state.get_filter() {
                        View::Done if !done => checkbox.hide(&ui),
                        View::Pending if done => checkbox.hide(&ui),
                        _ => checkbox.show(&ui),
                    }
                }

                // Set selected filter
                let view = VIEWS.iter().position(|&v| v == state.get_filter()).unwrap() as i64;
                filter.set_selected(&ui, view);
            }
        }
    });

    header.append(&ui, input, LayoutStrategy::Stretchy);
    header.append(&ui, add, LayoutStrategy::Compact);
    header.append(&ui, filter, LayoutStrategy::Compact);
    body.append(&ui, header, LayoutStrategy::Compact);
    body.append(&ui, HorizontalSeparator::new(&ui), LayoutStrategy::Compact);

    // The window allows all constituent components to be displayed.
    let mut window = Window::new(&ui, "Reducer <3 libui", 400, 300, WindowType::NoMenubar);
    window.set_child(&ui, body);
    window.show(&ui);

    event_loop.run(&ui);
}

#[cfg(feature = "async")]
fn main() -> Result<(), Box<dyn Error>> {
    // Create a channel to synchronize states.
    let (reactor, states) = channel();

    // Create a Store to manage the state.
    let store = AsyncStore::new(Arc::new(State::default()), reactor);

    // Listen for actions on a separate thread
    let dispatcher = store.spawn_thread()?;

    run_libui(dispatcher, states);

    Ok(())
}

#[cfg(not(feature = "async"))]
impl Dispatcher<Action> for std::sync::mpsc::Sender<Action> {
    type Output = Result<(), std::sync::mpsc::SendError<Action>>;

    fn dispatch(&mut self, action: Action) -> Self::Output {
        self.send(action)
    }
}

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn Error>> {
    // Create a channel to synchronize actions.
    let (dispatcher, actions) = channel();

    // Create a channel to synchronize states.
    let (reactor, states) = channel();

    // Run Reducer on a separate thread
    std::thread::spawn(move || {
        // Create a Store to manage the state.
        let mut store = Store::new(Arc::new(State::default()), reactor);

        // Listen for actions.
        while let Ok(action) = actions.recv() {
            store.dispatch(action).unwrap();
        }
    });

    run_libui(dispatcher, states);

    Ok(())
}
