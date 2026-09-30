use std::time::Duration;

use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    prelude::*,
    state::app::StatesPlugin,
};
use bevy_ratatui::{event::KeyMessage, RatatuiContext, RatatuiPlugins};
use color_eyre::Result;
use ratatui::{
    crossterm::event::{KeyCode, KeyEventKind},
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

mod dam;
mod river;
mod score;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

#[derive(Resource, Debug, Clone)]
struct Config {
    grid_width: u16,
    grid_height: u16,
    river_top: u16,
    dam_y: u16,
    river_width: u16,
    chunk_width: u16,
    chunk_count: usize,
    wave_speed: f32,
    meander_frequency: f32,
    meander_amplitude: f32,
    score_per_wave: u32,
    lives_initial: u32,
}

impl Config {
    fn chunk_slots(&self) -> usize {
        (self.grid_width / self.chunk_width) as usize
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            grid_width: 40,
            grid_height: 22,
            river_top: 1,
            dam_y: 18,
            river_width: 3,
            chunk_width: 4,
            chunk_count: 3,
            wave_speed: 1.0,
            meander_frequency: 4.0,
            meander_amplitude: 8.0,
            score_per_wave: 10,
            lives_initial: 3,
        }
    }
}

#[derive(Resource, Debug, Default)]
struct RiverState {
    wave: u32,
    progress: f32,
    seed: f32,
}

#[derive(Resource, Debug, Default)]
struct Dam {
    chunks: Vec<bool>,
}

#[derive(Resource, Debug, Default)]
struct Beaver {
    cursor: usize,
    holding: bool,
}

#[derive(Resource, Debug, Default)]
struct Score {
    score: u32,
    high_score: u32,
    lives: u32,
}

#[derive(Resource, Debug, Default)]
struct SoundQueue(Vec<SoundEvent>);

#[derive(Debug, Clone, Copy)]
enum SoundEvent {
    Pickup,
    Drop,
    Score,
    Leak,
    GameOver,
    Menu,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f32(
                1.0 / 30.0,
            ))),
            StatesPlugin,
            RatatuiPlugins::default(),
        ))
        .init_state::<GameState>()
        .insert_resource(Config::default())
        .insert_resource(Dam::default())
        .insert_resource(RiverState::default())
        .insert_resource(Beaver::default())
        .insert_resource(Score::default())
        .insert_resource(SoundQueue::default())
        .add_systems(PreUpdate, input_system)
        .add_systems(OnEnter(GameState::Playing), start_game)
        .add_systems(Update, update_river.run_if(in_state(GameState::Playing)))
        .add_systems(Update, draw_system)
        .run();

    Ok(())
}

fn start_game(
    mut score: ResMut<Score>,
    mut river: ResMut<RiverState>,
    mut dam: ResMut<Dam>,
    mut beaver: ResMut<Beaver>,
    config: Res<Config>,
    mut sound: ResMut<SoundQueue>,
) {
    score.score = 0;
    score.lives = config.lives_initial;
    river.wave = 1;
    river.progress = 0.0;
    // Start the first wave centred at the dam line.
    river.seed = -config.meander_frequency;
    let slots = config.chunk_slots();
    dam.chunks = vec![false; slots];
    let start = slots.saturating_sub(config.chunk_count) / 2;
    for i in start..start + config.chunk_count.min(slots) {
        dam.chunks[i] = true;
    }
    beaver.cursor = slots / 2;
    beaver.holding = false;
    sound.0.push(SoundEvent::Menu);
}

#[allow(clippy::too_many_arguments)]
fn input_system(
    mut messages: MessageReader<KeyMessage>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
    mut beaver: ResMut<Beaver>,
    mut dam: ResMut<Dam>,
    mut sound: ResMut<SoundQueue>,
    config: Res<Config>,
) {
    for message in messages.read() {
        let is_press = message.kind == KeyEventKind::Press;
        let is_repeat = message.kind == KeyEventKind::Repeat;

        match state.get() {
            GameState::Menu => match message.code {
                KeyCode::Enter | KeyCode::Char(' ') if is_press => {
                    next_state.set(GameState::Playing);
                }
                KeyCode::Char('q') | KeyCode::Esc if is_press => {
                    exit.write_default();
                }
                _ => {}
            },
            GameState::Playing => match message.code {
                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h')
                    if is_press || is_repeat =>
                {
                    beaver.cursor = beaver.cursor.saturating_sub(1);
                }
                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l')
                    if is_press || is_repeat =>
                {
                    if beaver.cursor + 1 < config.chunk_slots() {
                        beaver.cursor += 1;
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') if is_press => {
                    toggle_chunk(&mut beaver, &mut dam, &mut sound);
                }
                KeyCode::Esc if is_press => {
                    next_state.set(GameState::Menu);
                }
                _ => {}
            },
            GameState::GameOver => match message.code {
                KeyCode::Enter | KeyCode::Char(' ') if is_press => {
                    next_state.set(GameState::Playing);
                }
                KeyCode::Esc | KeyCode::Char('q') if is_press => {
                    next_state.set(GameState::Menu);
                }
                _ => {}
            },
        }
    }
}

fn toggle_chunk(beaver: &mut Beaver, dam: &mut Dam, sound: &mut SoundQueue) {
    if beaver.holding {
        if !dam.chunks[beaver.cursor] {
            dam.chunks[beaver.cursor] = true;
            beaver.holding = false;
            sound.0.push(SoundEvent::Drop);
        }
    } else if dam.chunks[beaver.cursor] {
        dam.chunks[beaver.cursor] = false;
        beaver.holding = true;
        sound.0.push(SoundEvent::Pickup);
    }
}

fn update_river(
    time: Res<Time>,
    config: Res<Config>,
    mut river: ResMut<RiverState>,
    mut score: ResMut<Score>,
    dam: Res<Dam>,
    mut next_state: ResMut<NextState<GameState>>,
    mut sound: ResMut<SoundQueue>,
) {
    let length = (config.dam_y - config.river_top) as f32;
    river.progress += config.wave_speed * time.delta_secs() / length;

    if river.progress >= 1.0 {
        river.progress = 0.0;

        let center = river::river_center(
            config.dam_y as f32,
            config.river_top as f32,
            config.dam_y as f32,
            river.seed,
            config.meander_frequency,
            config.meander_amplitude,
            config.grid_width,
        );
        let cols = river::river_columns(center, config.river_width, config.grid_width);
        let covered = dam::covers(&dam.chunks, config.chunk_width as usize, &cols);
        let result = score::evaluate_wave(
            score.score,
            score.lives,
            score.high_score,
            covered,
            config.score_per_wave,
        );

        score.score = result.score;
        score.lives = result.lives;
        score.high_score = result.high_score;

        if result.game_over {
            sound.0.push(SoundEvent::GameOver);
            next_state.set(GameState::GameOver);
        } else {
            sound.0.push(if covered {
                SoundEvent::Score
            } else {
                SoundEvent::Leak
            });
        }

        river.wave += 1;
        river.seed = river.wave as f32 * 1.618;
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_system(
    mut context: ResMut<RatatuiContext>,
    state: Res<State<GameState>>,
    config: Res<Config>,
    river: Res<RiverState>,
    dam: Res<Dam>,
    beaver: Res<Beaver>,
    score: Res<Score>,
    mut sound: ResMut<SoundQueue>,
) {
    for _ in sound.0.drain(..) {
        let _ = std::io::Write::write_all(&mut std::io::stdout(), b"\x07");
    }
    let _ = std::io::Write::flush(&mut std::io::stdout());

    context
        .draw(|frame| {
            let area = frame.area();
            frame.render_widget(
                Block::default().style(Style::default().bg(Color::Black)),
                area,
            );

            if area.width == 0 || area.height == 0 {
                return;
            }
            if area.width < config.grid_width || area.height < config.grid_height + 1 {
                frame.render_widget(
                    Paragraph::new("Terminal too small - resize to at least 40x19")
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Red)),
                    area,
                );
                return;
            }

            let playfield = center_rect(area, config.grid_width, config.grid_height + 1);

            match state.get() {
                GameState::Menu => draw_menu(frame, playfield, &score),
                GameState::Playing | GameState::GameOver => draw_game(
                    frame,
                    playfield,
                    &config,
                    &river,
                    &dam,
                    &beaver,
                    &score,
                    *state.get(),
                ),
            }
        })
        .expect("terminal draw failed");
}

fn center_rect(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

fn draw_menu(frame: &mut ratatui::Frame, area: Rect, score: &Score) {
    let text = Text::from(vec![
        Line::from("🦫 BEVYR 🦫").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::from("Keep the dam over the meandering river."),
        Line::from(""),
        Line::from("Controls:"),
        Line::from("  ← → / A D      move beaver"),
        Line::from("  Space / Enter  pick up / drop chunk"),
        Line::from("  Esc / Q        quit"),
        Line::from(""),
        Line::from(format!("High Score: {}", score.high_score)),
        Line::from(""),
        Line::from("Press Enter to start").style(Style::default().fg(Color::Green)),
    ]);
    let paragraph = Paragraph::new(text).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

#[allow(clippy::too_many_arguments)]
fn draw_game(
    frame: &mut ratatui::Frame,
    area: Rect,
    config: &Config,
    river: &RiverState,
    dam: &Dam,
    beaver: &Beaver,
    score: &Score,
    state: GameState,
) {
    let hud = Paragraph::new(Line::from(vec![Span::raw(format!(
        "Score: {}  High: {}  Lives: {}  Wave: {}",
        score.score, score.high_score, score.lives, river.wave
    ))]))
    .style(Style::default().fg(Color::White));
    frame.render_widget(
        hud,
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        },
    );

    let grid_top = area.y + 1;
    let front_row = config.river_top
        + ((config.dam_y - config.river_top) as f32 * river.progress).round() as u16;

    for dy in config.river_top..=config.dam_y {
        let center = river::river_center(
            dy as f32,
            config.river_top as f32,
            config.dam_y as f32,
            river.seed,
            config.meander_frequency,
            config.meander_amplitude,
            config.grid_width,
        );
        let cols = river::river_columns(center, config.river_width, config.grid_width);

        let is_front = dy == front_row;
        let is_past = dy < front_row;

        for col in cols {
            let x = area.x + col as u16;
            let y = grid_top + dy;
            let cell = &mut frame.buffer_mut()[(x, y)];
            if is_front {
                cell.set_symbol("▼");
                cell.set_fg(Color::Cyan);
                cell.set_bg(Color::Blue);
            } else if is_past {
                cell.set_symbol("~");
                cell.set_fg(Color::Blue);
                cell.set_bg(Color::Black);
            }
        }
    }

    let dam_y_abs = grid_top + config.dam_y;

    // Predictor: faint caret under where the current wave will hit the dam.
    let predicted_center = river::river_center(
        config.dam_y as f32,
        config.river_top as f32,
        config.dam_y as f32,
        river.seed,
        config.meander_frequency,
        config.meander_amplitude,
        config.grid_width,
    );
    for col in river::river_columns(predicted_center, config.river_width, config.grid_width) {
        let x = area.x + col as u16;
        let cell = &mut frame.buffer_mut()[(x, dam_y_abs)];
        cell.set_symbol("^");
        cell.set_fg(Color::DarkGray);
        cell.set_bg(Color::Black);
    }

    // Draw the three dam chunks.
    for slot in 0..config.chunk_slots() {
        if dam.chunks[slot] {
            let left = slot * config.chunk_width as usize;
            let right = left + config.chunk_width as usize;
            for col in left..right {
                let x = area.x + col as u16;
                let cell = &mut frame.buffer_mut()[(x, dam_y_abs)];
                cell.set_symbol("#");
                cell.set_fg(Color::Rgb(139, 69, 19));
                cell.set_bg(Color::Black);
            }
        }
    }

    // Beaver cursor / held chunk ghost.
    let cursor_left = beaver.cursor * config.chunk_width as usize;
    for offset in 0..config.chunk_width as usize {
        let col = cursor_left + offset;
        let x = area.x + col as u16;
        let cell = &mut frame.buffer_mut()[(x, dam_y_abs)];
        if offset == 0 {
            cell.set_symbol(if beaver.holding { "B" } else { "b" });
            cell.set_fg(Color::Yellow);
            cell.set_bg(Color::Black);
        } else if beaver.holding {
            cell.set_symbol("#");
            cell.set_fg(Color::Rgb(180, 120, 60));
            cell.set_bg(Color::Black);
        }
    }

    if state == GameState::GameOver {
        let popup = Paragraph::new("GAME OVER\nPress Enter to restart\nEsc for menu")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            );
        let popup_area = center_rect(area, 32, 5);
        frame.render_widget(popup, popup_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dam_with_chunks(slots: usize, occupied: &[usize]) -> Dam {
        let mut dam = Dam {
            chunks: vec![false; slots],
        };
        for &slot in occupied {
            dam.chunks[slot] = true;
        }
        dam
    }

    #[test]
    fn pick_up_chunk_removes_it_and_holds() {
        let mut beaver = Beaver {
            cursor: 2,
            holding: false,
        };
        let mut dam = dam_with_chunks(10, &[2, 5]);
        let mut sound = SoundQueue::default();

        toggle_chunk(&mut beaver, &mut dam, &mut sound);

        assert!(beaver.holding);
        assert!(!dam.chunks[2]);
        assert!(dam.chunks[5]);
        assert_eq!(sound.0.len(), 1);
        assert!(matches!(sound.0[0], SoundEvent::Pickup));
    }

    #[test]
    fn drop_chunk_places_it_and_releases() {
        let mut beaver = Beaver {
            cursor: 4,
            holding: true,
        };
        let mut dam = dam_with_chunks(10, &[5]);
        let mut sound = SoundQueue::default();

        toggle_chunk(&mut beaver, &mut dam, &mut sound);

        assert!(!beaver.holding);
        assert!(dam.chunks[4]);
        assert!(dam.chunks[5]);
        assert_eq!(sound.0.len(), 1);
        assert!(matches!(sound.0[0], SoundEvent::Drop));
    }

    #[test]
    fn drop_on_occupied_slot_does_nothing() {
        let mut beaver = Beaver {
            cursor: 5,
            holding: true,
        };
        let mut dam = dam_with_chunks(10, &[5]);
        let mut sound = SoundQueue::default();

        toggle_chunk(&mut beaver, &mut dam, &mut sound);

        assert!(beaver.holding);
        assert!(dam.chunks[5]);
        assert!(sound.0.is_empty());
    }
}
