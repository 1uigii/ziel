use ratatui::{style, widgets::canvas};

pub trait ShapeExt {
    fn draw_ext(self, painter: &mut canvas::Painter);
}

pub trait ContextExt {
    fn draw_ext<S: ShapeExt>(&mut self, shape: S);
    fn draw_ext_batch<S: ShapeExt, I: Iterator<Item = S>>(&mut self, iter: I) {
        for item in iter {
            self.draw_ext(item);
        }
    }
}

impl<'a> ContextExt for canvas::Context<'a> {
    fn draw_ext<S: ShapeExt>(&mut self, shape: S) {
        shape.draw_ext(&mut canvas::Painter::from(self));
    }
    fn draw_ext_batch<S: ShapeExt, I: Iterator<Item = S>>(&mut self, iter: I) {
        let mut painter = canvas::Painter::from(self);
        for item in iter {
            item.draw_ext(&mut painter);
        }
    }
}

impl ShapeExt for logic::ship::Ship {
    fn draw_ext(self, painter: &mut canvas::Painter) {
        for pos in self {
            let (x, y) = pos.to_coords();
            painter.paint(x as usize, y as usize, style::Color::White);
        }
    }
}

impl ShapeExt for (logic::ship::Ship, style::Color) {
    fn draw_ext(self, painter: &mut canvas::Painter) {
        for pos in self.0 {
            let (x, y) = pos.to_coords();
            painter.paint(x as usize, y as usize, self.1);
        }
    }
}

impl ShapeExt for &[[Option<client::AttackInfo>; 10]; 10] {
    fn draw_ext(self, painter: &mut canvas::Painter) {
        for ((x, y), info) in self
            .into_iter()
            .enumerate()
            .flat_map(|(i, info)| Iterator::zip(std::iter::repeat(i), info.into_iter().enumerate()))
            .filter_map(|(y, (x, info))| info.map(|i| ((x, y), i)))
        {
            painter.paint(
                x,
                y,
                match info {
                    client::AttackInfo::Hit => style::Color::LightRed,
                    client::AttackInfo::Miss => style::Color::Gray,
                },
            )
        }
    }
}

impl ShapeExt for (logic::Position, style::Color) {
    fn draw_ext(self, painter: &mut canvas::Painter) {
        let (x, y) = self.0.to_coords();
        painter.paint(x as usize, y as usize, self.1);
    }
}
