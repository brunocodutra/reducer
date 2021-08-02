//! A simple example demonstrating how to implement a Todo List app using Reducer & Conrod.

use conrod_core::color::{Colorable, DARK_CHARCOAL, LIGHT_BLUE, WHITE};
use conrod_core::position::{Positionable, Sizeable};
use conrod_core::text::Font;
use conrod_core::widget::{text_box, Button, Canvas, List, TextBox, Toggle, Widget};
use conrod_core::{image, widget_ids, Borderable, Labelable, UiBuilder, UiCell};
use conrod_glium::Renderer;
use conrod_winit::v023_conversion_fns;

use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::{dpi::LogicalSize, window::WindowBuilder, ContextBuilder};
use glium::{Display, Surface};

use reducer::{Dispatcher, Reactor, Reducer, Store};
use ring_channel::{ring_channel, RingReceiver};
use smol::{block_on, spawn};
use std::time::{Duration, Instant};
use std::{error::Error, mem, num::NonZeroUsize, sync::Arc};

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

v023_conversion_fns!();

// Renders the widgets given the current state.
fn render<E: Error + 'static>(
    ui: &mut UiCell,
    ids: &Ids,
    state: &State,
    dispatcher: &mut impl Dispatcher<Action, Output = Result<(), E>>,
) -> Result<(), Box<dyn Error>> {
    // Setup the layout.
    Canvas::new()
        .wh_of(ui.window)
        .pad(20.0)
        .pad_top(80.0)
        .pad_bottom(80.0)
        .middle_of(ui.window)
        .color(DARK_CHARCOAL)
        .set(ids.body, ui);

    Canvas::new()
        .h(80.0)
        .pad(20.0)
        .pad_right(130.0)
        .mid_top_of(ui.window)
        .color(DARK_CHARCOAL)
        .border_color(DARK_CHARCOAL)
        .set(ids.control, ui);

    Canvas::new()
        .h(80.0)
        .w_of(ui.window)
        .pad(20.0)
        .mid_bottom_of(ui.window)
        .color(DARK_CHARCOAL)
        .border_color(DARK_CHARCOAL)
        .set(ids.footer, ui);

    // Render button to add a todo.
    for _ in Button::new()
        .top_right_with_margin_on(ui.window, 20.0)
        .w(100.0)
        .kid_area_h_of(ids.footer)
        .label("Add Todo")
        .set(ids.button, ui)
    {
        dispatcher.dispatch(Action::AddTodo)?;
    }

    // Render text input.
    for input in TextBox::new(state.get_input())
        .mid_left_of(ids.control)
        .kid_area_wh_of(ids.control)
        .left_justify()
        .set(ids.input, ui)
    {
        if let text_box::Event::Update(text) = input {
            dispatcher.dispatch(Action::EditTodo(text))?;
        }
    }

    let todos = state.get_todos();

    // Render the list of todos.
    let (mut items, scrollbar) = List::flow_down(todos.len())
        .item_size(40.0)
        .scrollbar_on_top()
        .middle_of(ids.body)
        .kid_area_wh_of(ids.body)
        .set(ids.list, ui);

    while let Some(item) = items.next(ui) {
        let (idx, done, todo) = todos[item.i];
        let toggle = Toggle::new(!done)
            .label(todo)
            .label_color(WHITE)
            .color(LIGHT_BLUE);

        for _ in item.set(toggle, ui) {
            dispatcher.dispatch(Action::ToggleTodo(idx))?;
        }
    }

    // Enable scrolling if necessary.
    if let Some(s) = scrollbar {
        s.set(ui)
    }

    // Render button to show all todos.
    let mut all = Button::new()
        .mid_left_of(ids.footer)
        .w(120.0)
        .kid_area_h_of(ids.footer)
        .label("All")
        .enabled(state.get_filter() != View::All);

    if state.get_filter() == View::All {
        let color = WHITE.highlighted();
        all = all.color(color).hover_color(color).press_color(color);
    }

    for _ in all.set(ids.all, ui) {
        dispatcher.dispatch(Action::FilterTodos(View::All))?;
    }

    // Render button to show only completed todos.
    let mut done = Button::new()
        .middle_of(ids.footer)
        .w(120.0)
        .kid_area_h_of(ids.footer)
        .label("Done")
        .enabled(state.get_filter() != View::Done);

    if state.get_filter() == View::Done {
        let color = WHITE.highlighted();
        done = done.color(color).hover_color(color).press_color(color);
    }

    for _ in done.set(ids.done, ui) {
        dispatcher.dispatch(Action::FilterTodos(View::Done))?;
    }

    // Render button to show only pending todos.
    let mut pending = Button::new()
        .mid_right_of(ids.footer)
        .w(120.0)
        .kid_area_h_of(ids.footer)
        .label("Pending")
        .enabled(state.get_filter() != View::Pending);

    if state.get_filter() == View::Pending {
        let color = WHITE.highlighted();
        pending = pending.color(color).hover_color(color).press_color(color);
    }

    for _ in pending.set(ids.pending, ui) {
        dispatcher.dispatch(Action::FilterTodos(View::Pending))?;
    }

    Ok(())
}

fn run<E: Error + 'static>(
    mut dispatcher: impl Dispatcher<Action, Output = Result<(), E>> + 'static,
    mut states: RingReceiver<Arc<State>>,
) -> Result<(), Box<dyn Error>> {
    const WIDTH: f64 = 400.;
    const HEIGHT: f64 = 500.;

    let event_loop = EventLoop::new();
    let context = ContextBuilder::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::<f64>::from((WIDTH, HEIGHT)))
        .with_resizable(false)
        .with_title("Reducer <3 Conrod");

    let display = Display::new(window, context, &event_loop)?;
    let mut renderer = Renderer::new(&display)?;

    let font = Font::from_bytes(ttf_noto_sans::REGULAR)?;
    let mut ui = UiBuilder::new([WIDTH, HEIGHT]).build();
    ui.fonts.insert(font);

    let ids = Ids::new(ui.widget_id_generator());
    let image_map = image::Map::<glium::Texture2d>::new();

    // Keep a copy of the current state.
    let mut state = Arc::new(State::default());

    event_loop.run(move |event, _, ctrl| {
        // Stop the event loop when the window is closed.
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *ctrl = ControlFlow::Exit;
            return;
        }

        // Process window events.
        if let Some(event) = convert_event(&event, display.gl_window().window()) {
            ui.handle_event(event);
        }

        // Update the current state.
        if let Ok(next) = states.try_recv() {
            state = next;
        }

        // Render widgets given the current state.
        render(&mut ui.set_widgets(), &ids, &state, &mut dispatcher).unwrap();

        // Draw the `Ui` if it has changed.
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }

        // Render at ~60fps.
        *ctrl = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16));
    });
}

fn main() -> Result<(), Box<dyn Error>> {
    // Create a channel that always holds the latest state.
    let (tx, rx) = ring_channel(NonZeroUsize::new(1).unwrap());

    // Create a Store to manage the state.
    let store = Store::new(
        Arc::new(State::default()),
        <dyn Reactor<Arc<State>, Error = _>>::from_sink(tx),
    );

    // Turn store into an asynchronous task
    let (task, dispatcher) = store.into_task();

    // Spawn the asynchronous task on a background thread.
    let handle = spawn(task);

    // Run the rendering loop.
    run(dispatcher, rx)?;

    // Wait for the background thread to complete.
    block_on(handle)?;

    Ok(())
}
