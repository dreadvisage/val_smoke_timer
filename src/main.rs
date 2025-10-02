#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;

use config::{
    Config, InputBinding, button_to_string, get_all_buttons, get_all_keys, key_to_string,
};
use display_info::DisplayInfo;
use eframe::{App, Error, NativeOptions};
use egui::{
    CentralPanel, Color32, ComboBox, Context, DragValue, FontId, IconData, Margin, Rect, Rgba,
    ScrollArea, Sense, Slider, ViewportBuilder, ViewportCommand, Visuals, viewport::WindowLevel,
};
use rdev::{Button, Event, EventType, Key, listen};
use std::sync::{
    Arc,
    mpsc::{self, Receiver},
};
use std::time::{Duration, Instant};

static APP_TITLE: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

fn main() -> Result<(), Error> {
    let config = Config::load();

    let window_icon = load_icon();

    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_app_id(env!("CARGO_PKG_NAME"))
            .with_inner_size([400.0, 500.0])
            .with_resizable(false)
            .with_transparent(true)
            .with_icon(window_icon.clone()),
        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(move |_cc| Ok(Box::new(MainApp::new(config)))),
    )
}

fn load_icon() -> Arc<IconData> {
    let icon_bytes = include_bytes!("../assets/icon_256x256.png");

    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon from memory")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    Arc::new(IconData {
        rgba,
        width,
        height,
    })
}

// Main app that manages state transitions
enum AppState {
    Config(ConfigState),
    Timer(TimerState),
}

struct MainApp {
    state: AppState,
}

impl MainApp {
    fn new(config: Config) -> Self {
        Self {
            state: AppState::Config(ConfigState::new(config)),
        }
    }
}

impl App for MainApp {
    fn clear_color(&self, visuals: &Visuals) -> [f32; 4] {
        match &self.state {
            AppState::Config(_) => visuals.panel_fill.to_normalized_gamma_f32(),
            AppState::Timer(_) => Rgba::TRANSPARENT.to_array(),
        }
    }

    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        match &mut self.state {
            AppState::Config(config_state) => {
                if let Some(config) = config_state.update(ctx, frame) {
                    // Transition to timer state
                    // First reconfigure the viewport
                    ctx.send_viewport_cmd(ViewportCommand::Title(APP_TITLE.to_string()));
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize([1000.0, 200.0].into()));
                    ctx.send_viewport_cmd(ViewportCommand::Resizable(false));
                    ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
                    ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
                    ctx.send_viewport_cmd(ViewportCommand::OuterPosition(
                        [config.initial_pos.0, config.initial_pos.1].into(),
                    ));

                    self.state = AppState::Timer(TimerState::new(config));

                    ctx.request_repaint();
                }
            }
            AppState::Timer(timer_state) => {
                timer_state.update(ctx, frame);
                ctx.request_repaint();
            }
        }
    }
}

// Configuration State
#[derive(PartialEq)]
enum InputType {
    Keyboard,
    Mouse,
}

struct ConfigState {
    config: Config,
    start_input_type: InputType,
    start_key_selected: usize,
    start_button_selected: usize,
    confirm_input_type: InputType,
    confirm_key_selected: usize,
    confirm_button_selected: usize,
    available_keys: Vec<Key>,
    available_buttons: Vec<Button>,
    cancelable_keys_selected: Vec<usize>,
    cancelable_buttons_selected: Vec<usize>,
}

impl ConfigState {
    fn new(config: Config) -> Self {
        let available_keys = get_all_keys();
        let available_buttons = get_all_buttons();

        // Parse start key/button
        let start_binding = InputBinding::from_string(&config.start_key);
        let (start_input_type, start_key_selected, start_button_selected) = match start_binding {
            Some(InputBinding::Key(key)) => {
                let idx = available_keys
                    .iter()
                    .position(|k| key_to_string(k) == key_to_string(&key))
                    .unwrap_or(0);
                (InputType::Keyboard, idx, 0)
            }
            Some(InputBinding::Mouse(button)) => {
                let idx = available_buttons
                    .iter()
                    .position(|b| button_to_string(b) == button_to_string(&button))
                    .unwrap_or(0);
                (InputType::Mouse, 0, idx)
            }
            None => (InputType::Keyboard, 0, 0),
        };

        // Parse confirm key/button
        let confirm_binding = InputBinding::from_string(&config.confirm_key);
        let (confirm_input_type, confirm_key_selected, confirm_button_selected) =
            match confirm_binding {
                Some(InputBinding::Key(key)) => {
                    let idx = available_keys
                        .iter()
                        .position(|k| key_to_string(k) == key_to_string(&key))
                        .unwrap_or(0);
                    (InputType::Keyboard, idx, 0)
                }
                Some(InputBinding::Mouse(button)) => {
                    let idx = available_buttons
                        .iter()
                        .position(|b| button_to_string(b) == button_to_string(&button))
                        .unwrap_or(0);
                    (InputType::Mouse, 0, idx)
                }
                None => (InputType::Mouse, 0, 2), // Default to right mouse
            };

        // Find indices of cancelable keys and buttons
        let mut cancelable_keys_selected = Vec::new();
        let mut cancelable_buttons_selected = Vec::new();

        for key_str in &config.cancelable_keys {
            if let Some(binding) = InputBinding::from_string(key_str) {
                match binding {
                    InputBinding::Key(k) => {
                        if let Some(idx) = available_keys
                            .iter()
                            .position(|key| key_to_string(key) == key_to_string(&k))
                        {
                            cancelable_keys_selected.push(idx);
                        }
                    }
                    InputBinding::Mouse(b) => {
                        if let Some(idx) = available_buttons
                            .iter()
                            .position(|button| button_to_string(button) == button_to_string(&b))
                        {
                            cancelable_buttons_selected.push(idx);
                        }
                    }
                }
            }
        }

        Self {
            config,
            start_input_type,
            start_key_selected,
            start_button_selected,
            confirm_input_type,
            confirm_key_selected,
            confirm_button_selected,
            available_keys,
            available_buttons,
            cancelable_keys_selected,
            cancelable_buttons_selected,
        }
    }

    // Returns Some(config) when ready to transition to timer
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) -> Option<Config> {
        let mut should_start = false;
        let mut should_reset = false;

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Timer Configuration");
            ui.add_space(10.0);

            ScrollArea::vertical().show(ui, |ui| {
                // Allocate remaining space to force full width
                ui.allocate_space(egui::vec2(ui.available_width(), 0.0));

                // Initial Position
                ui.group(|ui| {
                    ui.label("Initial Window Position");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(DragValue::new(&mut self.config.initial_pos.0).speed(1.0));
                        ui.label("Y:");
                        ui.add(DragValue::new(&mut self.config.initial_pos.1).speed(1.0));
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Set to Primary Monitor Top-Left").clicked() {
                            match DisplayInfo::all() {
                                Ok(displays) => {
                                    // Try to find primary display
                                    if let Some(primary) = displays.iter().find(|d| d.is_primary) {
                                        self.config.initial_pos =
                                            (primary.x as f32, primary.y as f32);
                                    } else if let Some(first) = displays.first() {
                                        // Fallback to first display
                                        self.config.initial_pos = (first.x as f32, first.y as f32);
                                    } else {
                                        // Final fallback
                                        self.config.initial_pos = (0.0, 0.0);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to get display info: {e:?}");
                                    self.config.initial_pos = (0.0, 0.0);
                                }
                            }
                        }
                    });
                });
                ui.add_space(10.0);

                // Start Key/Button
                ui.group(|ui| {
                    ui.push_id("start_input", |ui| {
                        ui.label("Start Key/Button (first input in sequence)");
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut self.start_input_type,
                                InputType::Keyboard,
                                "Keyboard Key",
                            );
                            ui.radio_value(
                                &mut self.start_input_type,
                                InputType::Mouse,
                                "Mouse Button",
                            );
                        });

                        match self.start_input_type {
                            InputType::Keyboard => {
                                ComboBox::from_id_salt("start_key_combo")
                                    .selected_text(key_to_string(
                                        &self.available_keys[self.start_key_selected],
                                    ))
                                    .show_ui(ui, |ui| {
                                        for (i, key) in self.available_keys.iter().enumerate() {
                                            ui.selectable_value(
                                                &mut self.start_key_selected,
                                                i,
                                                key_to_string(key),
                                            );
                                        }
                                    });
                                self.config.start_key =
                                    InputBinding::Key(self.available_keys[self.start_key_selected])
                                        .to_string();
                            }
                            InputType::Mouse => {
                                ComboBox::from_id_salt("start_button_combo")
                                    .selected_text(button_to_string(
                                        &self.available_buttons[self.start_button_selected],
                                    ))
                                    .show_ui(ui, |ui| {
                                        for (i, button) in self.available_buttons.iter().enumerate()
                                        {
                                            ui.selectable_value(
                                                &mut self.start_button_selected,
                                                i,
                                                button_to_string(button),
                                            );
                                        }
                                    });
                                self.config.start_key = InputBinding::Mouse(
                                    self.available_buttons[self.start_button_selected],
                                )
                                .to_string();
                            }
                        }
                    });
                });
                ui.add_space(10.0);

                // Confirm Key/Button
                ui.group(|ui| {
                    ui.push_id("confirm_input", |ui| {
                        ui.label("Confirm Key/Button (second input in sequence)");
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut self.confirm_input_type,
                                InputType::Keyboard,
                                "Keyboard Key",
                            );
                            ui.radio_value(
                                &mut self.confirm_input_type,
                                InputType::Mouse,
                                "Mouse Button",
                            );
                        });

                        match self.confirm_input_type {
                            InputType::Keyboard => {
                                ComboBox::from_id_salt("confirm_key_combo")
                                    .selected_text(key_to_string(
                                        &self.available_keys[self.confirm_key_selected],
                                    ))
                                    .show_ui(ui, |ui| {
                                        for (i, key) in self.available_keys.iter().enumerate() {
                                            ui.selectable_value(
                                                &mut self.confirm_key_selected,
                                                i,
                                                key_to_string(key),
                                            );
                                        }
                                    });
                                self.config.confirm_key = InputBinding::Key(
                                    self.available_keys[self.confirm_key_selected],
                                )
                                .to_string();
                            }
                            InputType::Mouse => {
                                ComboBox::from_id_salt("confirm_button_combo")
                                    .selected_text(button_to_string(
                                        &self.available_buttons[self.confirm_button_selected],
                                    ))
                                    .show_ui(ui, |ui| {
                                        for (i, button) in self.available_buttons.iter().enumerate()
                                        {
                                            ui.selectable_value(
                                                &mut self.confirm_button_selected,
                                                i,
                                                button_to_string(button),
                                            );
                                        }
                                    });
                                self.config.confirm_key = InputBinding::Mouse(
                                    self.available_buttons[self.confirm_button_selected],
                                )
                                .to_string();
                            }
                        }
                    });
                });
                ui.add_space(10.0);

                // Cancelable Keys/Buttons
                ui.group(|ui| {
                    ui.label("Cancelable Inputs (keys/buttons that reset the sequence)");
                    ui.label("Select multiple inputs:");

                    // Add padding on the right by constraining the width
                    let available_width = ui.available_width();
                    ui.set_max_width(available_width - 15.0);

                    ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                        ui.label("Keyboard Keys:");
                        for (i, key) in self.available_keys.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let mut is_selected = self.cancelable_keys_selected.contains(&i);
                                if ui.checkbox(&mut is_selected, key_to_string(key)).changed() {
                                    if is_selected {
                                        if !self.cancelable_keys_selected.contains(&i) {
                                            self.cancelable_keys_selected.push(i);
                                        }
                                    } else {
                                        self.cancelable_keys_selected.retain(|&x| x != i);
                                    }
                                }
                                ui.allocate_space(egui::vec2(ui.available_width(), 0.0));
                            });
                        }

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.label("Mouse Buttons:");
                        for (i, button) in self.available_buttons.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let mut is_selected = self.cancelable_buttons_selected.contains(&i);
                                if ui
                                    .checkbox(&mut is_selected, button_to_string(button))
                                    .changed()
                                {
                                    if is_selected {
                                        if !self.cancelable_buttons_selected.contains(&i) {
                                            self.cancelable_buttons_selected.push(i);
                                        }
                                    } else {
                                        self.cancelable_buttons_selected.retain(|&x| x != i);
                                    }
                                }
                                ui.allocate_space(egui::vec2(ui.available_width(), 0.0));
                            });
                        }
                    });

                    // Update config with both keys and buttons
                    let mut cancelable = Vec::new();
                    for &i in &self.cancelable_keys_selected {
                        cancelable.push(InputBinding::Key(self.available_keys[i]).to_string());
                    }
                    for &i in &self.cancelable_buttons_selected {
                        cancelable.push(InputBinding::Mouse(self.available_buttons[i]).to_string());
                    }
                    self.config.cancelable_keys = cancelable;
                });
                ui.add_space(10.0);

                // Timer Start Duration
                ui.group(|ui| {
                    ui.label("Timer Duration (seconds)");
                    ui.add(Slider::new(&mut self.config.timer_start, 1.0..=99.99).text("seconds"));
                });
                ui.add_space(10.0);

                // Max Timers
                ui.group(|ui| {
                    ui.label("Maximum Active Timers");
                    ui.add(Slider::new(&mut self.config.max_timers, 1..=5).text("timers"));
                });
                ui.add_space(10.0);

                // Subtext Options
                ui.group(|ui| {
                    ui.label("Timer Display Options");
                    ui.checkbox(&mut self.config.show_subtext, "Show Subtext Label");
                    if self.config.show_subtext {
                        ui.horizontal(|ui| {
                            ui.label("Subtext:");
                            ui.text_edit_singleline(&mut self.config.subtext_string);
                        });
                    }
                    ui.checkbox(&mut self.config.show_numbering, "Show Timer Numbers (1-5)");
                });
                ui.add_space(10.0);

                // Timer Behavior Options
                ui.group(|ui| {
                    ui.label("Timer Behavior");
                    ui.checkbox(&mut self.config.add_new_on_left, "Add New Timers on Left");
                    if !self.config.add_new_on_left {
                        ui.label("(New timers will be added on the right)");
                    }
                    ui.checkbox(
                        &mut self.config.overwrite_oldest,
                        "Overwrite Oldest Timer When Full",
                    );
                    if !self.config.overwrite_oldest {
                        ui.label("(Will wait for free slot when at max timers)");
                    }
                });
                ui.add_space(10.0);

                // Red Text Warning Options
                ui.group(|ui| {
                    ui.checkbox(&mut self.config.enable_red_text, "Enable Red Text Warning");
                    if self.config.enable_red_text {
                        ui.horizontal(|ui| {
                            ui.label("Warning Threshold:");
                            ui.add(
                                DragValue::new(&mut self.config.red_text_threshold)
                                    .speed(0.1)
                                    .range(0.1..=self.config.timer_start)
                                    .suffix(" sec"),
                            );
                        });
                        ui.label("Text turns red when time remaining is below this threshold");
                    }
                });
                ui.add_space(20.0);

                // Buttons
                ui.horizontal(|ui| {
                    if ui.button("Save and Start").clicked() {
                        if let Err(e) = self.config.save() {
                            eprintln!("Failed to save config: {e:?}");
                        }
                        should_start = true;
                    }

                    if ui.button("Reset to Defaults").clicked() {
                        should_reset = true;
                    }

                    if ui.button("Cancel").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                });
            });
        });

        if should_reset {
            // Reset to defaults
            *self = ConfigState::new(Config::default());
            None
        } else if should_start {
            Some(self.config.clone())
        } else {
            None
        }
    }
}

// Timer State
#[derive(Clone, Copy, Debug, PartialEq)]
enum InputEvent {
    KeyPress(Key),
    MousePress(Button),
}

enum Command {
    StartTimer,
}

struct TimerState {
    config: Config,
    timers: Vec<Timer>,
    rx: Receiver<Command>,
}

struct Timer {
    end_time: Instant,
}

impl Timer {
    fn new(duration_ms: u64) -> Self {
        Self {
            end_time: Instant::now() + Duration::from_millis(duration_ms),
        }
    }

    fn remaining_ms(&self) -> i64 {
        let now = Instant::now();
        if now >= self.end_time {
            0
        } else {
            self.end_time.duration_since(now).as_millis() as i64
        }
    }

    fn is_finished(&self) -> bool {
        self.remaining_ms() <= 0
    }
}

struct SequenceDetector {
    waiting_for_confirm: bool,
    start_binding: InputBinding,
    confirm_binding: InputBinding,
    cancel_keys: Vec<Key>,
    cancel_buttons: Vec<Button>,
}

impl SequenceDetector {
    fn new(config: &Config) -> Self {
        let start_binding =
            InputBinding::from_string(&config.start_key).unwrap_or(InputBinding::Key(Key::KeyE));
        let confirm_binding = InputBinding::from_string(&config.confirm_key)
            .unwrap_or(InputBinding::Mouse(Button::Right));

        let mut cancel_keys = Vec::new();
        let mut cancel_buttons = Vec::new();

        for s in &config.cancelable_keys {
            if let Some(binding) = InputBinding::from_string(s) {
                match binding {
                    InputBinding::Key(k) => cancel_keys.push(k),
                    InputBinding::Mouse(b) => cancel_buttons.push(b),
                }
            }
        }

        Self {
            waiting_for_confirm: false,
            start_binding,
            confirm_binding,
            cancel_keys,
            cancel_buttons,
        }
    }

    fn on_input(&mut self, input: InputEvent) -> bool {
        match input {
            InputEvent::KeyPress(key) => {
                if !self.waiting_for_confirm {
                    if let InputBinding::Key(start_key) = &self.start_binding
                        && key_to_string(&key) == key_to_string(start_key)
                    {
                        self.waiting_for_confirm = true;
                    }
                } else if self
                    .cancel_keys
                    .iter()
                    .any(|k| key_to_string(k) == key_to_string(&key))
                {
                    self.waiting_for_confirm = false;
                } else if let InputBinding::Key(confirm_key) = &self.confirm_binding
                    && key_to_string(&key) == key_to_string(confirm_key)
                {
                    self.waiting_for_confirm = false;
                    return true;
                }
            }
            InputEvent::MousePress(button) => {
                if !self.waiting_for_confirm {
                    if let InputBinding::Mouse(start_button) = &self.start_binding
                        && button_to_string(&button) == button_to_string(start_button)
                    {
                        self.waiting_for_confirm = true;
                    }
                } else if self
                    .cancel_buttons
                    .iter()
                    .any(|b| button_to_string(b) == button_to_string(&button))
                {
                    self.waiting_for_confirm = false;
                } else if let InputBinding::Mouse(confirm_button) = &self.confirm_binding
                    && button_to_string(&button) == button_to_string(confirm_button)
                {
                    self.waiting_for_confirm = false;
                    return true;
                }
            }
        }
        false
    }
}

impl TimerState {
    fn new(config: Config) -> Self {
        let (tx, rx) = mpsc::channel();
        let config_clone = config.clone();

        std::thread::spawn(move || {
            let mut detector = SequenceDetector::new(&config_clone);

            if let Err(error) = listen(move |event: Event| match event.event_type {
                EventType::KeyPress(key) => {
                    if detector.on_input(InputEvent::KeyPress(key)) {
                        let _ = tx.send(Command::StartTimer);
                    }
                }
                EventType::ButtonPress(button) => {
                    if detector.on_input(InputEvent::MousePress(button)) {
                        let _ = tx.send(Command::StartTimer);
                    }
                }
                _ => {}
            }) {
                eprintln!("Error listening to events: {error:?}");
            }
        });

        Self {
            config,
            timers: Vec::new(),
            rx,
        }
    }

    fn format_time(ms: i64) -> String {
        let seconds = ms / 1000;
        let millis = (ms % 1000) / 10;
        format!("{seconds:02}:{millis:02}")
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(true));

        while let Ok(command) = self.rx.try_recv() {
            match command {
                Command::StartTimer => {
                    let duration_ms = (self.config.timer_start * 1000.0) as u64;

                    if self.timers.len() < self.config.max_timers {
                        // We have space, add the timer
                        if self.config.add_new_on_left {
                            self.timers.insert(0, Timer::new(duration_ms));
                        } else {
                            self.timers.push(Timer::new(duration_ms));
                        }
                    } else if self.config.overwrite_oldest {
                        // At capacity but configured to overwrite
                        if self.config.add_new_on_left {
                            self.timers.pop(); // Remove oldest (rightmost)
                            self.timers.insert(0, Timer::new(duration_ms));
                        } else {
                            self.timers.remove(0); // Remove oldest (leftmost)
                            self.timers.push(Timer::new(duration_ms));
                        }
                    }
                    // else: at capacity and not overwriting, do nothing (wait for free slot)
                }
            }
        }

        self.timers.retain(|timer| !timer.is_finished());

        CentralPanel::default()
            .frame(egui::Frame {
                fill: Color32::TRANSPARENT,
                inner_margin: Margin::same(0),
                outer_margin: Margin::same(0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                if !self.timers.is_empty() {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 30.0;

                        for (i, timer) in self.timers.iter().enumerate() {
                            ui.vertical(|ui| {
                                let remaining = timer.remaining_ms();
                                let time_str = Self::format_time(remaining);

                                let text_color = if self.config.enable_red_text
                                    && remaining <= (self.config.red_text_threshold * 1000.0) as i64
                                {
                                    Color32::RED
                                } else {
                                    Color32::WHITE
                                };

                                let font_id = FontId::monospace(48.0);
                                let galley = ui.fonts(|f| {
                                    f.layout_no_wrap(time_str.clone(), font_id.clone(), text_color)
                                });
                                let text_size = galley.size();

                                let (rect, _) = ui.allocate_exact_size(text_size, Sense::hover());

                                ui.painter().rect_filled(
                                    rect.expand(10.0),
                                    5.0,
                                    Color32::from_rgba_unmultiplied(0, 0, 0, 180),
                                );

                                ui.painter().galley(rect.left_top(), galley, text_color);

                                if self.config.show_subtext || self.config.show_numbering {
                                    let smoke_number = if self.config.add_new_on_left {
                                        self.timers.len() - i
                                    } else {
                                        i + 1
                                    };

                                    let mut subtext_parts = Vec::new();
                                    if self.config.show_subtext
                                        && !self.config.subtext_string.is_empty()
                                    {
                                        subtext_parts.push(self.config.subtext_string.clone());
                                    }
                                    if self.config.show_numbering {
                                        subtext_parts.push(smoke_number.to_string());
                                    }

                                    let subtext = subtext_parts.join(" ");

                                    if !subtext.is_empty() {
                                        let subtext_font_id = FontId::monospace(12.0);
                                        let subtext_galley = ui.fonts(|f| {
                                            f.layout_no_wrap(
                                                subtext.clone(),
                                                subtext_font_id.clone(),
                                                Color32::WHITE,
                                            )
                                        });
                                        let subtext_size = subtext_galley.size();

                                        let timer_width = text_size.x;
                                        let subtext_width = subtext_size.x;
                                        let x_offset = (timer_width - subtext_width) / 2.0;

                                        let (subtext_rect, _) = ui.allocate_exact_size(
                                            egui::vec2(timer_width, subtext_size.y),
                                            Sense::hover(),
                                        );

                                        let centered_subtext_rect = Rect::from_min_size(
                                            egui::pos2(
                                                subtext_rect.min.x + x_offset,
                                                subtext_rect.min.y,
                                            ),
                                            subtext_size,
                                        );

                                        ui.painter().rect_filled(
                                            centered_subtext_rect.expand(5.0),
                                            3.0,
                                            Color32::from_rgba_unmultiplied(0, 0, 0, 180),
                                        );

                                        ui.painter().galley(
                                            centered_subtext_rect.left_top(),
                                            subtext_galley,
                                            Color32::WHITE,
                                        );
                                    }
                                }
                            });
                        }
                    });
                }
            });

        ctx.request_repaint();
    }
}
