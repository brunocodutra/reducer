//! A simple example demonstrating how to implement a Todo List app using Reducer & egui.

use eframe::egui::{CentralPanel, Context, Key, ScrollArea, TopBottomPanel};
use eframe::{epaint::vec2, run_native, App, Frame, NativeOptions};
use reducer::{AsyncReactor, Dispatcher, Reducer, Store};
use ring_channel::{ring_channel, RingReceiver};
use std::{error::Error, mem, num::NonZeroUsize, sync::Arc};
use tokio::task::spawn;

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

// The actions our users can trigger.
#[derive(Debug, Clone)]
enum Action {
    AddTodo,
    EditTodo(String),
    ToggleTodo(usize),
    FilterTodos(View),
}

// Our app's state.
#[derive(Debug, Default, Clone)]
struct State {
    input: String,
    todos: Vec<(bool, String)>,
    filter: View,
}

impl State {
    // These associated functions project the state into derived
    // properties that are more convenient for rendering.

    fn get_input(&self) -> &str {
        &self.input
    }

    fn get_filter(&self) -> View {
        self.filter
    }

    fn get_todos(&self) -> Vec<(usize, bool, &str)> {
        self.todos
            .iter()
            .enumerate()
            .map(|(i, &(done, ref todo))| (i, done, todo.as_str()))
            .filter(|(_, done, _)| match self.filter {
                View::All => true,
                View::Done => *done,
                View::Pending => !*done,
            })
            .collect()
    }
}

impl Reducer<Action> for State {
    // Our app's business logic goes here.
    fn reduce(&mut self, action: Action) {
        match action {
            Action::AddTodo => {
                if !self.input.is_empty() {
                    let todo = mem::replace(&mut self.input, "".into());
                    self.todos.push((false, todo));
                }
            }

            Action::EditTodo(text) => {
                self.input = text;
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

struct Application<D: Dispatcher<Action>> {
    state: Arc<State>,
    receiver: RingReceiver<Arc<State>>,
    dispatcher: D,
}

impl<D: Dispatcher<Action>> Application<D> {
    fn new(receiver: RingReceiver<Arc<State>>, dispatcher: D) -> Self {
        Application {
            state: Default::default(),
            receiver,
            dispatcher,
        }
    }
}

impl<D: Dispatcher<Action>> App for Application<D> {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        // Receive our app's latest state.
        if let Ok(next) = self.receiver.try_recv() {
            self.state = next;
        }

        // Render the widgets.
        TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                let mut input = self.state.get_input().to_string();
                let text_edit = ui.text_edit_singleline(&mut input);

                if text_edit.changed() {
                    self.dispatcher.dispatch(Action::EditTodo(input));
                }

                if ui.button("Add todo").clicked()
                    || (text_edit.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)))
                {
                    self.dispatcher.dispatch(Action::AddTodo);
                }

                text_edit.request_focus();
            });
        });

        TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                let filter = self.state.get_filter();

                if ui.toggle_value(&mut (filter == View::All), "All").clicked() {
                    self.dispatcher.dispatch(Action::FilterTodos(View::All));
                }

                if ui
                    .toggle_value(&mut (filter == View::Done), "Done")
                    .clicked()
                {
                    self.dispatcher.dispatch(Action::FilterTodos(View::Done));
                }

                if ui
                    .toggle_value(&mut (filter == View::Pending), "Pending")
                    .clicked()
                {
                    self.dispatcher.dispatch(Action::FilterTodos(View::Pending));
                }
            })
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    for (id, done, todo) in self.state.get_todos() {
                        if ui.toggle_value(&mut (!done), todo).clicked() {
                            self.dispatcher.dispatch(Action::ToggleTodo(id));
                        }
                    }
                });
            });
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a channel that always holds the latest state.
    let (tx, rx) = ring_channel(NonZeroUsize::new(1).unwrap());

    // Create a Store to manage the state.
    let store = Store::new(Arc::new(State::default()), AsyncReactor(tx));

    // Turn store into an asynchronous task
    let (task, dispatcher) = store.into_task();

    // Spawn the asynchronous task on a background thread.
    let handle = spawn(task);

    // Run egui.
    run_native(
        "reducer <3 egui",
        NativeOptions {
            resizable: false,
            initial_window_size: Some(vec2(380., 500.)),
            ..NativeOptions::default()
        },
        Box::new(|_| Box::new(Application::new(rx, dispatcher))),
    );

    // Wait for the background thread to complete.
    handle.await??;

    Ok(())
}
