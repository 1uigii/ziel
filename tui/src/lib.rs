use std::io;

use crossterm::event::{self, KeyCode};
use ratatui::{
    layout,
    style::{self, Stylize},
    symbols, text,
    widgets::{self, canvas},
};

mod draw_ext;
use draw_ext::ContextExt;

const SHIP_COLORS: [style::Color; 5] = [
    style::Color::from_u32(0xffcdb2),
    style::Color::from_u32(0xffb4a2),
    style::Color::from_u32(0xe5989b),
    style::Color::from_u32(0xb5838d),
    style::Color::from_u32(0x6d6875),
];

#[allow(unused)]
struct Layout<'s> {
    pub client_board: layout::Rect,
    pub opponent_board: layout::Rect,
    pub messages: layout::Rect,
    pub help: layout::Rect,

    pub client_board_border: widgets::Block<'s>,
    pub opponent_board_border: widgets::Block<'s>,
}

fn message_to_line(message: client::ui::Message) -> Option<text::Line<'static>> {
    match message {
        client::ui::Message::OpponentSelectsTarget => None,
        client::ui::Message::ClientMissedOpponent => Some(text::Line::from(vec![
            text::Span::raw("your shot "),
            text::Span::raw("missed").light_red(),
        ])),
        client::ui::Message::OpponentMissedClient => Some(text::Line::from(vec![
            text::Span::raw("opps shot "),
            text::Span::raw("missed").yellow(),
        ])),
        client::ui::Message::ClientHitOpponent => Some(text::Line::from(vec![
            text::Span::raw("your shot "),
            text::Span::raw("hit").yellow(),
        ])),
        client::ui::Message::OpponentHitClient => Some(text::Line::from(vec![
            text::Span::raw("opps shot "),
            text::Span::raw("hit").light_red(),
        ])),
        client::ui::Message::OpponentShipSunk => Some(text::Line::from(vec![
            text::Span::raw("opps ship "),
            text::Span::raw("sunk").yellow(),
        ])),
        client::ui::Message::ClientShipSunk => Some(text::Line::from(vec![
            text::Span::raw("your ship "),
            text::Span::raw("sunk").light_red(),
        ])),
    }
}

impl<'s> Layout<'s> {
    fn generate(area: layout::Rect) -> Layout<'s> {
        let [_left, middle, _right] = layout::Layout::horizontal([
            layout::Constraint::Fill(1),
            layout::Constraint::Length(23),
            layout::Constraint::Fill(1),
        ])
        .areas(area);

        let [help, boards, messages] = layout::Layout::vertical([
            layout::Constraint::Fill(1),
            layout::Constraint::Length(7),
            layout::Constraint::Fill(1),
        ])
        .areas(middle);

        let [client_board, opponent_board] = layout::Layout::horizontal([
            layout::Constraint::Length(11),
            layout::Constraint::Length(12),
        ])
        .areas(boards);

        let client_board_border = widgets::Block::bordered()
            .border_type(widgets::BorderType::Thick)
            .borders(widgets::Borders::ALL ^ widgets::Borders::RIGHT);

        let opponent_board_border = widgets::Block::bordered()
            .border_type(widgets::BorderType::Thick)
            .border_set(symbols::border::Set {
                top_left: symbols::line::THICK_HORIZONTAL_DOWN,
                bottom_left: symbols::line::THICK_HORIZONTAL_UP,
                ..symbols::border::THICK
            });

        Layout {
            client_board,
            opponent_board,
            messages,
            help,
            client_board_border,
            opponent_board_border,
        }
    }

    fn paint_client_board<F>(&self, f: &mut ratatui::Frame, paint_fn: F)
    where
        F: Fn(&mut canvas::Context),
    {
        f.render_widget(
            Tui::new_board_canvas(self.client_board_border.clone()).paint(|ctx| {
                ctx.draw(&canvas::Points {
                    coords: &[(0.0, 0.0)],
                    color: style::Color::White,
                });
                paint_fn(ctx);
            }),
            self.client_board,
        );
    }

    fn paint_opponent_board<F>(&self, f: &mut ratatui::Frame, paint_fn: F)
    where
        F: Fn(&mut canvas::Context),
    {
        f.render_widget(
            Tui::new_board_canvas(self.opponent_board_border.clone()).paint(|ctx| {
                ctx.draw(&canvas::Points {
                    coords: &[(0.0, 0.0)],
                    color: style::Color::White,
                });
                paint_fn(ctx);
            }),
            self.opponent_board,
        );
    }

    fn draw_messages(&self, f: &mut ratatui::Frame, messages: &[client::ui::Message]) {
        let iter: Vec<_> = messages
            .into_iter()
            .rev()
            .into_iter()
            .filter_map(|&m| message_to_line(m))
            .collect();

        f.render_widget(widgets::Paragraph::new(iter).gray(), self.messages);
    }
}

#[derive(Debug)]
pub struct Tui {
    term: ratatui::DefaultTerminal,
}

impl Tui {
    pub fn init() -> Tui {
        Tui {
            term: ratatui::init(),
        }
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("tui :: io :: {0}")]
    Io(#[from] io::Error),
    #[error("input :: player interrupt")]
    PlayerInterrupt,
}

impl client::UI for Tui {
    type Error = Error;

    fn request_ships(&mut self) -> Result<logic::Ships, Self::Error> {
        let mut x = 0u8;
        let mut y = 0u8;

        let mut ships = logic::Ships::try_from([
            logic::ship::Ship::try_from(logic::ship::ShipPlan::Vertical {
                pos: logic::Position::try_from_coords((0, 0)).unwrap(),
                length: 5,
            })
            .unwrap(),
            logic::ship::Ship::try_from(logic::ship::ShipPlan::Vertical {
                pos: logic::Position::try_from_coords((1, 0)).unwrap(),
                length: 4,
            })
            .unwrap(),
            logic::ship::Ship::try_from(logic::ship::ShipPlan::Vertical {
                pos: logic::Position::try_from_coords((2, 0)).unwrap(),
                length: 3,
            })
            .unwrap(),
            logic::ship::Ship::try_from(logic::ship::ShipPlan::Vertical {
                pos: logic::Position::try_from_coords((3, 0)).unwrap(),
                length: 3,
            })
            .unwrap(),
            logic::ship::Ship::try_from(logic::ship::ShipPlan::Vertical {
                pos: logic::Position::try_from_coords((4, 0)).unwrap(),
                length: 2,
            })
            .unwrap(),
        ])
        .unwrap();

        loop {
            match event::read()? {
                event::Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Char('a') | KeyCode::Left if x > 0 => x -= 1,
                        KeyCode::Char('d') | KeyCode::Right if x < 9 => x += 1,
                        KeyCode::Char('w') | KeyCode::Up if y > 0 => y -= 1,
                        KeyCode::Char('s') | KeyCode::Down if y < 9 => y += 1,
                        KeyCode::Char(' ') => {
                            let pos = logic::Position::try_from_coords((x, y)).unwrap();
                            for (idx, ship) in ships.into_iter().enumerate() {
                                if ship.into_iter().any(|p| p == pos) {
                                    ships = self.place_ship(ships, idx, &mut x, &mut y)?;
                                    break;
                                }
                            }
                        }
                        KeyCode::Enter => return Ok(ships),
                        KeyCode::Char('q') => return Err(Error::PlayerInterrupt),
                        _ => {}
                    }
                }
                _ => {}
            }

            self.term.draw(|f| {
                let [horizonta_area] = layout::Layout::horizontal([layout::Constraint::Length(12)])
                    .flex(layout::Flex::Center)
                    .areas(f.area());
                let [area] = layout::Layout::vertical([layout::Constraint::Length(7)])
                    .flex(layout::Flex::Center)
                    .areas(horizonta_area);

                let block = widgets::Block::bordered();

                let canvas = Tui::new_board_canvas(block).paint(|ctx| {
                    // internal variable dirty needs to be set to `true` D:<
                    ctx.draw(&canvas::Points {
                        coords: &[(-1.0, -1.0)],
                        color: style::Color::White,
                    });
                    ctx.draw_ext_batch(
                        ships
                            .into_iter()
                            .enumerate()
                            .map(|(i, ship)| (ship, SHIP_COLORS[i])),
                    );
                    ctx.draw_ext((
                        logic::Position::try_from_coords((x, y)).unwrap(),
                        style::Color::White,
                    ));
                });
                f.render_widget(canvas, area);
            })?;
        }
    }

    fn request_target(
        &mut self,
        info: client::ui::ClientInfo,
    ) -> Result<logic::Position, Self::Error> {
        let mut x = 0u8;
        let mut y = 0u8;

        loop {
            match event::read()? {
                event::Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Char('a') | KeyCode::Left if x > 0 => x -= 1,
                        KeyCode::Char('d') | KeyCode::Right if x < 9 => x += 1,
                        KeyCode::Char('w') | KeyCode::Up if y > 0 => y -= 1,
                        KeyCode::Char('s') | KeyCode::Down if y < 9 => y += 1,
                        KeyCode::Char(' ') => {
                            let pos = logic::Position::try_from_coords((x, y)).unwrap();
                            if info.opponent_hit_map[pos].is_none() {
                                return Ok(pos);
                            }
                        }
                        KeyCode::Char('q') => return Err(Error::PlayerInterrupt),
                        _ => {}
                    }
                }
                _ => {}
            }

            self.term.draw(|f| {
                let mut layout = Layout::generate(f.area());
                layout.opponent_board_border = layout.opponent_board_border.title("sel. targ.");

                layout.paint_client_board(f, |ctx| {
                    ctx.draw_ext_batch(
                        info.ships
                            .into_iter()
                            .cloned()
                            .enumerate()
                            .map(|(i, ship)| (ship, SHIP_COLORS[i])),
                    );
                    ctx.draw_ext(info.client_hit_map);
                });

                layout.paint_opponent_board(f, |ctx| {
                    ctx.draw_ext(info.opponent_hit_map);
                    ctx.draw_ext_batch(
                        info.opponent_ships
                            .into_iter()
                            .cloned()
                            .zip(std::iter::repeat(style::Color::Red)),
                    );
                    ctx.draw_ext((
                        logic::Position::try_from_coords((x, y)).unwrap(),
                        style::Color::White,
                    ));
                });

                layout.draw_messages(f, info.messages);
            })?;
        }
    }

    fn display_board(&mut self, info: client::ui::ClientInfo) -> Result<(), Self::Error> {
        // TODO: something against event stacking
        while event::poll(std::time::Duration::from_secs(0))? {
            match event::read()? {
                event::Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Char('q') => return Err(Error::PlayerInterrupt),
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        self.term.draw(|f| {
            let layout = Layout::generate(f.area());

            layout.paint_client_board(f, |ctx| {
                ctx.draw_ext_batch(
                    info.ships
                        .into_iter()
                        .cloned()
                        .enumerate()
                        .map(|(i, ship)| (ship, SHIP_COLORS[i])),
                );
                ctx.draw_ext(info.client_hit_map);
            });

            layout.paint_opponent_board(f, |ctx| {
                ctx.draw_ext(info.opponent_hit_map);
                ctx.draw_ext_batch(
                    info.opponent_ships
                        .into_iter()
                        .cloned()
                        .zip(std::iter::repeat(style::Color::Red)),
                );
            });

            layout.draw_messages(f, info.messages);
        })?;

        Ok(())
    }

    fn display_victory(&mut self, info: client::ui::ClientInfo) -> Result<(), Self::Error> {
        const MESSAGE: &str = "V I C T O R Y";

        self.term.draw(|f| {
            let layout = Layout::generate(f.area());

            layout.paint_client_board(f, |ctx| {
                ctx.draw_ext_batch(
                    info.ships
                        .into_iter()
                        .cloned()
                        .enumerate()
                        .map(|(i, ship)| (ship, SHIP_COLORS[i])),
                );
                ctx.draw_ext(info.client_hit_map);
            });

            layout.paint_opponent_board(f, |ctx| {
                ctx.draw_ext(info.opponent_hit_map);
                ctx.draw_ext_batch(
                    info.opponent_ships
                        .into_iter()
                        .cloned()
                        .zip(std::iter::repeat(style::Color::Red)),
                );
            });

            layout.draw_messages(f, info.messages);

            let [center_box] = layout::Layout::vertical([layout::Constraint::Length(1)])
                .flex(layout::Flex::Center)
                .areas(f.area());
            let [center_box] = layout::Layout::horizontal([layout::Constraint::Length(
                (MESSAGE.len() + 2) as u16,
            )])
            .flex(layout::Flex::Center)
            .areas(center_box);

            f.render_widget(
                widgets::Paragraph::new(MESSAGE)
                    .bold()
                    .centered()
                    .yellow()
                    .on_white(),
                center_box,
            );
        })?;

        loop {
            match event::read()? {
                event::Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn display_loss(&mut self, info: client::ui::ClientInfo) -> Result<(), Self::Error> {
        const MESSAGE: &str = "L O S S";

        self.term.draw(|f| {
            let layout = Layout::generate(f.area());

            layout.paint_client_board(f, |ctx| {
                ctx.draw_ext_batch(
                    info.ships
                        .into_iter()
                        .cloned()
                        .enumerate()
                        .map(|(i, ship)| (ship, SHIP_COLORS[i])),
                );
                ctx.draw_ext(info.client_hit_map);
            });

            layout.paint_opponent_board(f, |ctx| {
                ctx.draw_ext(info.opponent_hit_map);
                ctx.draw_ext_batch(
                    info.opponent_ships
                        .into_iter()
                        .cloned()
                        .zip(std::iter::repeat(style::Color::Red)),
                );
            });

            layout.draw_messages(f, info.messages);

            let [center_box] = layout::Layout::vertical([layout::Constraint::Length(1)])
                .flex(layout::Flex::Center)
                .areas(f.area());
            let [center_box] = layout::Layout::horizontal([layout::Constraint::Length(
                (MESSAGE.len() + 2) as u16,
            )])
            .flex(layout::Flex::Center)
            .areas(center_box);

            f.render_widget(
                widgets::Paragraph::new(MESSAGE)
                    .bold()
                    .centered()
                    .cyan()
                    .on_white(),
                center_box,
            );
        })?;

        loop {
            match event::read()? {
                event::Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

impl Tui {
    fn new_board_canvas<'a, F: Fn(&mut canvas::Context)>(
        block: widgets::Block<'a>,
    ) -> canvas::Canvas<'a, F> {
        canvas::Canvas::<'a, F>::default()
            .block(block)
            .marker(symbols::Marker::HalfBlock)
    }

    fn place_ship(
        &mut self,
        ships: logic::Ships,
        ship_idx: usize,
        x: &mut u8,
        y: &mut u8,
    ) -> Result<logic::Ships, <Tui as client::UI>::Error> {
        let mut ships = ships.into_ship_array();
        let (ship_offset, ship_length, mut horizontal) = match ships[ship_idx].to_ship_plan() {
            logic::ship::ShipPlan::Horizontal { pos, length } => {
                (*x - pos.to_coords().0, length, true)
            }
            logic::ship::ShipPlan::Vertical { pos, length } => {
                (*y - pos.to_coords().1, length, false)
            }
        };

        loop {
            let mut check = false;
            match event::read()? {
                event::Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Char('a') | KeyCode::Left => *x = x.saturating_sub(1),
                        KeyCode::Char('d') | KeyCode::Right => *x += 1,
                        KeyCode::Char('w') | KeyCode::Up => *y = y.saturating_sub(1),
                        KeyCode::Char('s') | KeyCode::Down => *y += 1,
                        KeyCode::Char('r') => horizontal ^= true,
                        KeyCode::Char(' ') => check = true,
                        KeyCode::Char('q') => return Err(Error::PlayerInterrupt),
                        _ => {}
                    }
                }

                _ => {}
            }

            *x = u8::clamp(
                *x,
                if horizontal { ship_offset } else { 0 },
                if horizontal {
                    10 - ship_length + ship_offset
                } else {
                    9
                },
            );
            *y = u8::clamp(
                *y,
                if !horizontal { ship_offset } else { 0 },
                if !horizontal {
                    10 - ship_length + ship_offset
                } else {
                    9
                },
            );

            ships[ship_idx] = if horizontal {
                logic::ship::ShipPlan::Horizontal {
                    pos: logic::Position::try_from_coords((*x - ship_offset, *y)).unwrap(),
                    length: ship_length,
                }
            } else {
                logic::ship::ShipPlan::Vertical {
                    pos: logic::Position::try_from_coords((*x, *y - ship_offset)).unwrap(),
                    length: ship_length,
                }
            }
            .try_into()
            .unwrap();

            let valid = ships.try_into();

            if check {
                if let Ok(ships) = valid {
                    return Ok(ships);
                }
            }

            self.term.draw(|f| {
                let [horizonta_area] = layout::Layout::horizontal([layout::Constraint::Length(12)])
                    .flex(layout::Flex::Center)
                    .areas(f.area());
                let [area] = layout::Layout::vertical([layout::Constraint::Length(7)])
                    .flex(layout::Flex::Center)
                    .areas(horizonta_area);

                let block =
                    widgets::Block::bordered().style(style::Style::new().fg(if valid.is_ok() {
                        SHIP_COLORS[ship_idx]
                    } else {
                        style::Color::Red
                    }));

                let canvas = Tui::new_board_canvas(block).paint(|ctx| {
                    // internal variable dirty needs to be set to `true` D:<
                    ctx.draw(&canvas::Points {
                        coords: &[(-1.0, -1.0)],
                        color: style::Color::White,
                    });
                    ctx.draw_ext_batch(
                        ships
                            .into_iter()
                            .enumerate()
                            .map(|(i, ship)| (ship, SHIP_COLORS[i])),
                    );
                    ctx.draw_ext((ships[ship_idx], SHIP_COLORS[ship_idx]));
                    ctx.draw_ext((
                        logic::Position::try_from_coords((*x, *y)).unwrap(),
                        style::Color::White,
                    ));
                });
                f.render_widget(canvas, area);
            })?;
        }
    }
}
