//! A simple example demonstrating how to implement a Todo app using Reducer & Conrod.

#[macro_use]
extern crate conrod;
extern crate reducer;
extern crate ttf_noto_sans;

use conrod::backend::glium::{glium, Renderer};
use conrod::backend::winit;
use conrod::text::FontCollection;
use conrod::{color, image, UiBuilder, UiCell};
use conrod::{widget, Borderable, Colorable, Labelable, Positionable, Sizeable, Widget};
use glium::{glutin, texture, Display, Surface};
use glutin::Event::WindowEvent;
use glutin::WindowEvent::CloseRequested;
use glutin::{ContextBuilder, EventsLoop, WindowBuilder};
use reducer::*;
use std::error::Error;
use std::mem;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

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

// The actions a user can trigger on our app.
#[derive(Debug, Clone)]
enum Action {
    AddTodo,
    EditTodo(String),
    ToggleTodo(usize),
    FilterTodos(View),
}

// The state of our app.
#[derive(Debug, Default, Clone)]
struct State {
    input: String,
    todos: Vec<(bool, String)>,
    filter: View,
}

impl Reducer<Action> for State {
    // The entire business logic of our app goes here.
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

impl State {
    // These associated functions project the state into derived
    // properties that are more convenient for rendering.

    fn get_input(&self) -> &String {
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

// Register widgets.
widget_ids!(struct Ids { control, body, footer, button, input, list, all, done, pending });

// Renders the widgets given the current state.
fn render(ui: &mut UiCell, ids: &Ids, state: &State, mut dispatcher: impl Dispatcher<Action>) {
    // Setup the layout.
    widget::Canvas::new()
        .wh_of(ui.window)
        .pad(20.0)
        .pad_top(80.0)
        .pad_bottom(80.0)
        .middle_of(ui.window)
        .color(color::DARK_CHARCOAL)
        .set(ids.body, ui);

    widget::Canvas::new()
        .h(80.0)
        .pad(20.0)
        .pad_right(130.0)
        .mid_top_of(ui.window)
        .rgba(0.0, 0.0, 0.0, 0.0)
        .border_rgba(0.0, 0.0, 0.0, 0.0)
        .set(ids.control, ui);

    widget::Canvas::new()
        .h(80.0)
        .w_of(ui.window)
        .pad(20.0)
        .mid_bottom_of(ui.window)
        .rgba(0.0, 0.0, 0.0, 0.0)
        .border_rgba(0.0, 0.0, 0.0, 0.0)
        .set(ids.footer, ui);

    // Render button to add a todo.
    for _ in widget::Button::new()
        .top_right_with_margin_on(ui.window, 20.0)
        .w(100.0)
        .kid_area_h_of(ids.footer)
        .label("Add Todo")
        .set(ids.button, ui)
    {
        dispatcher.dispatch(Action::AddTodo);
    }

    // Render text input.
    for input in widget::TextBox::new(state.get_input())
        .mid_left_of(ids.control)
        .kid_area_wh_of(ids.control)
        .left_justify()
        .set(ids.input, ui)
    {
        if let widget::text_box::Event::Update(text) = input {
            dispatcher.dispatch(Action::EditTodo(text));
        }
    }

    let todos = state.get_todos();

    // Render the list of todos.
    let (mut items, scrollbar) = widget::List::flow_down(todos.len())
        .item_size(40.0)
        .scrollbar_on_top()
        .middle_of(ids.body)
        .kid_area_wh_of(ids.body)
        .set(ids.list, ui);

    while let Some(item) = items.next(ui) {
        let (idx, done, todo) = todos[item.i];
        let toggle = widget::Toggle::new(!done)
            .label(todo)
            .label_color(color::WHITE)
            .color(color::LIGHT_BLUE);

        for _ in item.set(toggle, ui) {
            dispatcher.dispatch(Action::ToggleTodo(idx));
        }
    }

    // Enable scrolling if necessary.
    if let Some(s) = scrollbar {
        s.set(ui)
    }

    // Render button to show all todos.
    let mut all = widget::Button::new()
        .mid_left_of(ids.footer)
        .w(120.0)
        .kid_area_h_of(ids.footer)
        .label("All")
        .enabled(state.get_filter() != View::All);

    if state.get_filter() == View::All {
        let color = color::WHITE.highlighted();
        all = all.color(color).hover_color(color).press_color(color);
    }

    for _ in all.set(ids.all, ui) {
        dispatcher.dispatch(Action::FilterTodos(View::All));
    }

    // Render button to show only completed todos.
    let mut done = widget::Button::new()
        .middle_of(ids.footer)
        .w(120.0)
        .kid_area_h_of(ids.footer)
        .label("Done")
        .enabled(state.get_filter() != View::Done);

    if state.get_filter() == View::Done {
        let color = color::WHITE.highlighted();
        done = done.color(color).hover_color(color).press_color(color);
    }

    for _ in done.set(ids.done, ui) {
        dispatcher.dispatch(Action::FilterTodos(View::Done));
    }

    // Render button to show only pending todos.
    let mut pending = widget::Button::new()
        .mid_right_of(ids.footer)
        .w(120.0)
        .kid_area_h_of(ids.footer)
        .label("Pending")
        .enabled(state.get_filter() != View::Pending);

    if state.get_filter() == View::Pending {
        let color = color::WHITE.highlighted();
        pending = pending.color(color).hover_color(color).press_color(color);
    }

    for _ in pending.set(ids.pending, ui) {
        dispatcher.dispatch(Action::FilterTodos(View::Pending));
    }
}

fn run_conrod(
    dispatcher: impl Dispatcher<Action> + Clone,
    states: Receiver<Arc<State>>,
) -> Result<(), Box<dyn Error>> {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 500;

    let mut events_loop = EventsLoop::new();
    let context = ContextBuilder::new();
    let window = WindowBuilder::new()
        .with_dimensions((WIDTH, HEIGHT).into())
        .with_min_dimensions((WIDTH, HEIGHT).into())
        .with_title("Reducer <3 Conrod");

    let display = Display::new(window, context, &events_loop)?;
    let mut ui = UiBuilder::new([f64::from(WIDTH), f64::from(HEIGHT)]).build();

    ui.fonts
        .insert(FontCollection::from_bytes(ttf_noto_sans::REGULAR)?.into_font()?);

    let ids = Ids::new(ui.widget_id_generator());
    let mut renderer = Renderer::new(&display)?;
    let image_map = image::Map::<texture::Texture2d>::new();

    // Keep a copy of the current state.
    let mut state = Arc::new(State::default());

    // Point in time of the last refresh.
    let mut rerendered_at = Instant::now();

    loop {
        let mut exit = false;

        // Rerender at 60fps.
        while Instant::now().duration_since(rerendered_at) < Duration::from_millis(16) {
            events_loop.poll_events(|event| {
                if let WindowEvent { ref event, .. } = event {
                    if let CloseRequested = event {
                        exit = true;
                    }
                }

                if let Some(input) = winit::convert_event(event, &display) {
                    ui.handle_event(input);
                }
            });

            // Avoid spinning too fast.
            thread::sleep(Duration::from_millis(1));
        }

        rerendered_at = Instant::now();

        if exit {
            break Ok(());
        }

        // Synchronize the state.
        if let Some(next) = states.try_iter().last() {
            state = next;
        }

        // Render widgets given the current state.
        render(&mut ui.set_widgets(), &ids, &state, dispatcher.clone());

        // Draw the `Ui` if it has changed.
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 0.0);
            renderer.draw(&display, &mut target, &image_map)?;
            target.finish()?;
        }
    }
}

#[cfg(feature = "async")]
fn main() -> Result<(), Box<dyn Error>> {
    // Create a channel to synchronize states.
    let (reactor, states) = channel();

    // Create a Store to manage the state.
    let store = AsyncStore::new(Arc::new(State::default()), reactor);

    // Listen for actions on a separate thread
    let dispatcher = store.spawn_thread()?;

    run_conrod(dispatcher, states)
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
    thread::spawn(move || {
        // Create a Store to manage the state.
        let mut store = Store::new(Arc::new(State::default()), reactor);

        // Listen for actions.
        while let Ok(action) = actions.recv() {
            store.dispatch(action).unwrap();
        }
    });

    run_conrod(dispatcher, states)
}
