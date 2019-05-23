//! A simple example demonstrating how to implement a Todo List app using Reducer & Conrod.

#[macro_use]
extern crate conrod_core;
extern crate conrod_gfx;
extern crate conrod_winit;
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate reducer;
extern crate ttf_noto_sans;

use conrod_core::text::FontCollection;
use conrod_core::{
    color, image, widget, Borderable, Colorable, Labelable, Positionable, Sizeable, UiBuilder,
    UiCell, Widget,
};
use conrod_gfx::{ColorFormat, Renderer};
use conrod_winit::convert_event;
use gfx::format::DepthStencil;
use gfx::Device;
use glutin::dpi::LogicalSize;
use glutin::Event::WindowEvent;
use glutin::WindowEvent::{CloseRequested, Resized};
use glutin::{ContextBuilder, EventsLoop, WindowBuilder};
use reducer::*;
use std::error::Error;
use std::mem;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
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
        .color(color::DARK_CHARCOAL)
        .border_color(color::DARK_CHARCOAL)
        .set(ids.control, ui);

    widget::Canvas::new()
        .h(80.0)
        .w_of(ui.window)
        .pad(20.0)
        .mid_bottom_of(ui.window)
        .color(color::DARK_CHARCOAL)
        .border_color(color::DARK_CHARCOAL)
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
    const WIDTH: f64 = 400.;
    const HEIGHT: f64 = 500.;

    let mut events_loop = EventsLoop::new();
    let context = ContextBuilder::new();
    let builder = WindowBuilder::new()
        .with_dimensions((WIDTH, HEIGHT).into())
        .with_min_dimensions((WIDTH, HEIGHT).into())
        .with_title("Reducer <3 Conrod");

    let (window, mut device, mut factory, rtv, _) =
        gfx_window_glutin::init::<ColorFormat, DepthStencil>(builder, context, &events_loop)?;

    let mut encoder = factory.create_command_buffer().into();
    let mut renderer = Renderer::new(&mut factory, &rtv, window.get_hidpi_factor())?;

    let font = FontCollection::from_bytes(ttf_noto_sans::REGULAR)?.into_font()?;
    let mut ui = UiBuilder::new([WIDTH, HEIGHT]).build();
    ui.fonts.insert(font);

    let ids = Ids::new(ui.widget_id_generator());
    let image_map = image::Map::new();

    // Keep a copy of the current state.
    let mut state = Arc::new(State::default());

    // Point in time of the last refresh.
    let mut rerendered_at = Instant::now();

    loop {
        // If the window is closed, this will be None for one tick, so to avoid panicking with
        // unwrap, instead break the loop
        let LogicalSize { width, height } = match window.get_inner_size() {
            Some(s) => s,
            None => break Ok(()),
        };

        // Draw the `Ui` if it has changed.
        if let Some(primitives) = ui.draw_if_changed() {
            let dpi_factor = window.get_hidpi_factor();
            let dims = ((width * dpi_factor) as f32, (height * dpi_factor) as f32);

            renderer.clear(&mut encoder, color::DARK_CHARCOAL.to_fsa());
            renderer.fill(&mut encoder, dims, dpi_factor, primitives, &image_map);
            renderer.draw(&mut factory, &mut encoder, &image_map);
            encoder.flush(&mut device);
            window.swap_buffers()?;
            device.cleanup();
        }

        let mut exit = false;

        events_loop.poll_events(|event| {
            if let WindowEvent { ref event, .. } = event {
                if let CloseRequested = event {
                    exit = true;
                }

                if let Resized(logical_size) = event {
                    let hidpi_factor = window.get_hidpi_factor();
                    let physical_size = logical_size.to_physical(hidpi_factor);
                    window.resize(physical_size);
                    let (new_color, _) =
                        gfx_window_glutin::new_views::<ColorFormat, DepthStencil>(&window);
                    renderer.on_resize(new_color);
                }
            }

            if let Some(event) = convert_event(event.clone(), window.window()) {
                ui.handle_event(event);
            }
        });

        if exit {
            break Ok(());
        }

        // Render at 60fps.
        if Instant::now().duration_since(rerendered_at) > Duration::from_millis(16) {
            // Synchronize the state.
            if let Some(next) = states.try_iter().last() {
                state = next;
            }

            // Render widgets given the current state.
            render(&mut ui.set_widgets(), &ids, &state, dispatcher.clone());
            rerendered_at = Instant::now();
        }
    }
}

// Use Reducer's experimental support for async/await.
#[cfg(feature = "async")]
fn main() -> Result<(), Box<dyn Error>> {
    // Create a channel to synchronize states.
    let (reactor, states) = channel();

    // Create a Store to manage the state.
    let store = Store::new(Arc::new(State::default()), reactor);

    // Spin up a thread-pool to run our application
    let mut executor = futures::executor::ThreadPool::new()?;

    // Listen for actions on a separate thread
    let dispatcher = executor.spawn_dispatcher(store).unwrap();

    // Spin up the rendering thread
    executor.run(futures::future::lazy(|_| run_conrod(dispatcher, states)))
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

    run_conrod(dispatcher, states)
}
