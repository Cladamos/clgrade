use crate::image::ScopeData;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    symbols::{self},
    text::Span,
    widgets::{
        Axis, Block,
        BorderType::Rounded,
        Chart, Dataset, GraphType, Widget,
        canvas::{Canvas, Circle, Line, Points},
    },
};

pub struct ScopeSection<'a> {
    scope_data: &'a ScopeData,
}

impl<'a> ScopeSection<'a> {
    pub const SCOPE_HEIGHT: u16 = 16;
    pub const LUM_HISTOGRAM_WIDTH: u16 = 32;
    pub const VECTORSCOPE_WIDTH: u16 = 32;

    pub fn new(scope_data: &'a ScopeData) -> Self {
        ScopeSection { scope_data }
    }
}

//TODO: add horizontal layout mode to scopes
impl<'a> Widget for ScopeSection<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let constraints =
            if area.width > ScopeSection::VECTORSCOPE_WIDTH + ScopeSection::LUM_HISTOGRAM_WIDTH {
                vec![
                    Constraint::Length(area.width - ScopeSection::VECTORSCOPE_WIDTH),
                    Constraint::Length(ScopeSection::VECTORSCOPE_WIDTH),
                ]
            } else {
                vec![Constraint::Percentage(50), Constraint::Percentage(50)]
            };
        let scopes_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);
        let max_count = self
            .scope_data
            .lum_histogram
            .iter()
            .copied()
            .max()
            .unwrap_or(1) as f64; // Fallback to 1 to prevent division by zero

        let luma_histogram = self
            .scope_data
            .lum_histogram
            .iter()
            .enumerate()
            .map(|(i, &x)| (i as f64, x as f64))
            .collect::<Vec<(f64, f64)>>();

        let dataset = Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .data(&luma_histogram);

        let x_axis = Axis::default()
            .style(Style::default().dark_gray())
            .bounds([0.0, 255.0])
            .labels(["0", "128", "255"]);

        let y_axis = Axis::default()
            .title("Pixel Count")
            .style(Style::default().dark_gray())
            .bounds([0.0, max_count])
            .labels(vec![
                Span::raw("0"),
                Span::raw(format!("{}", (max_count / 2.0) as u64)),
                Span::raw(format!("{}", max_count as u64)),
            ]);

        Block::bordered()
            .title("Luma Histogram (x: 0 pure black, x: 255 pure white)")
            .border_type(Rounded)
            .render(scopes_layout[0], buf);
        Chart::new(vec![dataset])
            .x_axis(x_axis)
            .y_axis(y_axis)
            .render(scopes_layout[0].inner(Margin::new(4, 2)), buf);

        let vectorscope = Canvas::default()
            .block(Block::bordered().title("Vectorscope").border_type(Rounded))
            .marker(symbols::Marker::Braille)
            .x_bounds([-1.2, 1.2])
            .y_bounds([-1.2, 1.2])
            .paint(|ctx| {
                ctx.draw(&Line {
                    x1: -1.0,
                    y1: 0.0,
                    x2: 1.0,
                    y2: 0.0,
                    color: Color::DarkGray,
                });
                ctx.draw(&Line {
                    x1: 0.0,
                    y1: -1.0,
                    x2: 0.0,
                    y2: 1.0,
                    color: Color::DarkGray,
                });

                ctx.draw(&Circle {
                    x: 0.0,
                    y: 0.0,
                    radius: 1.0,
                    color: Color::DarkGray,
                });

                ctx.draw(&Points {
                    coords: &self
                        .scope_data
                        .vectorscope_points
                        .iter()
                        .map(|&(x, y)| (x as f64, y as f64))
                        .collect::<Vec<(f64, f64)>>(),
                    color: Color::White,
                });
            });
        vectorscope.render(scopes_layout[1], buf);
    }
}
