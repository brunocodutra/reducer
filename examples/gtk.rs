//! A simple example demonstrating how to implement a Todo List app using Reducer & GTK.

extern crate gtk;
extern crate reducer;

use gtk::prelude::*;
use gtk::{Button, CheckButton, ComboBoxText, Entry, Orientation, Separator, Window, WindowType};
use reducer::*;
use std::cell::RefCell;
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

fn run_gtk(dispatcher: impl Dispatcher<Action> + Clone + 'static, states: Receiver<Arc<State>>) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("Reducer <3 GTK");
    window.set_default_size(400, 500);
    window.set_border_width(10);

    // Layout.
    let body = gtk::Box::new(Orientation::Vertical, 10);
    window.add(&body);

    let header = gtk::Box::new(Orientation::Horizontal, 5);
    body.pack_start(&header, false, false, 0);

    let separator = Separator::new(Orientation::Horizontal);
    body.add(&separator);

    // Text input.
    let input = Entry::new();
    header.pack_start(&input, true, true, 0);

    // "Add Todo" button.
    let button = Button::new_with_label("Add Todo");
    header.add(&button);

    button.connect_clicked({
        let dispatcher = RefCell::new(dispatcher.clone());

        move |_| {
            let buffer = input.get_buffer();
            let action = Action::AddTodo(buffer.get_text());
            dispatcher.borrow_mut().dispatch(action);
            buffer.set_text("");
        }
    });

    // View control
    const VIEWS: [View; 3] = [View::All, View::Done, View::Pending];
    let filter = ComboBoxText::new();

    for view in &VIEWS {
        filter.append_text(view.name());
    }

    filter.connect_changed({
        let dispatcher = RefCell::new(dispatcher.clone());

        move |this| {
            let action = Action::FilterTodos(VIEWS[this.get_active() as usize]);
            dispatcher.borrow_mut().dispatch(action);
        }
    });

    filter.set_active(0);
    header.add(&filter);

    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    idle_add({
        let mut checklist: Vec<CheckButton> = vec![];
        let body = body.clone();
        let filter = filter.clone();
        let dispatcher = dispatcher.clone();

        move || {
            // Update widgets on state change.
            if let Some(state) = states.try_iter().last() {
                // Add new todos
                let todos = state.get_todos();
                for (i, (_, todo)) in todos.iter().enumerate().skip(checklist.len()) {
                    let checkbutton = CheckButton::new_with_label(todo);
                    checkbutton.connect_toggled({
                        let dispatcher = RefCell::new(dispatcher.clone());
                        move |_| {
                            let action = Action::ToggleTodo(i);
                            dispatcher.borrow_mut().dispatch(action);
                        }
                    });
                    body.add(&checkbutton);
                    checklist.push(checkbutton);
                    body.show_all();
                }

                // Synchronize checklist with the state.
                for (&(done, _), checkbutton) in todos.iter().zip(checklist.iter_mut()) {
                    checkbutton.set_active(done);
                    match state.get_filter() {
                        View::Done if !done => checkbutton.hide(),
                        View::Pending if done => checkbutton.hide(),
                        _ => checkbutton.show(),
                    }
                }

                // Set selected filter
                let view = VIEWS.iter().position(|&v| v == state.get_filter()).unwrap() as i32;
                filter.set_active(view);
            }

            Continue(true)
        }
    });

    gtk::main();
}

// Use Reducer's experimental support for async/await.
#[cfg(feature = "async")]
fn main() -> Result<(), Box<dyn Error>> {
    // Create a channel to synchronize states.
    let (reactor, states) = channel();

    // Create a Store to manage the state.
    let store = Async::new(Store::new(Arc::new(State::default()), reactor));

    // Spin up a thread-pool to run our application
    let mut executor = futures::executor::ThreadPoolBuilder::new().create()?;

    // Listen for actions on a separate thread
    let dispatcher = store.spawn(&mut executor).unwrap();

    // Spin up the rendering thread
    executor.run(futures::future::lazy(|_| run_gtk(dispatcher, states)));

    Ok(())
}

// Fallback to features available in stable Rust.
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

    run_gtk(dispatcher, states);

    Ok(())
}
