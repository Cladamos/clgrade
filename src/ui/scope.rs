use crate::{app::AppLayout, image::ScopeData};

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
    app_layout: AppLayout,
}

impl<'a> ScopeSection<'a> {
    pub const SCOPE_HEIGHT: u16 = 15;
    pub const LUM_HISTOGRAM_WIDTH: u16 = 30;
    pub const VECTORSCOPE_WIDTH: u16 = 30;

    // +10 for the horizontal gap between sides
    pub const MIN_WIDTH: u16 = Self::LUM_HISTOGRAM_WIDTH + Self::VECTORSCOPE_WIDTH + 10;

    pub fn new(scope_data: &'a ScopeData, app_layout: AppLayout) -> Self {
        ScopeSection {
            scope_data,
            app_layout,
        }
    }
}

impl<'a> Widget for ScopeSection<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (scope_area, direction, constraints) = match self.app_layout {
            AppLayout::Vertical => (
                area,
                Direction::Horizontal,
                if area.width > Self::VECTORSCOPE_WIDTH + Self::LUM_HISTOGRAM_WIDTH {
                    vec![
                        Constraint::Length(area.width.saturating_sub(Self::VECTORSCOPE_WIDTH)),
                        Constraint::Length(Self::VECTORSCOPE_WIDTH),
                    ]
                } else {
                    vec![Constraint::Percentage(50), Constraint::Percentage(50)]
                },
            ),
            AppLayout::Horizontal => (
                Rect {
                    x: area.x,
                    y: area.y,
                    width: if area.width < Self::VECTORSCOPE_WIDTH {
                        area.width
                    } else {
                        Self::VECTORSCOPE_WIDTH
                    },
                    height: area.height.saturating_sub(2),
                },
                Direction::Vertical,
                if area.width < Self::VECTORSCOPE_WIDTH {
                    let vectorscope_height = area.width.saturating_div(2);
                    let histogram_height = area.height.saturating_sub(vectorscope_height);
                    vec![
                        Constraint::Length(histogram_height),
                        Constraint::Length(vectorscope_height),
                    ]
                } else {
                    vec![
                        Constraint::Length(area.height.saturating_sub(Self::SCOPE_HEIGHT)),
                        Constraint::Length(Self::SCOPE_HEIGHT),
                    ]
                },
            ),
        };
        let scopes_layout = Layout::default()
            .direction(direction)
            .constraints(constraints)
            .split(scope_area);
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
            // Swap x and y if layout is horizontal
            .map(if self.app_layout == AppLayout::Vertical {
                |(i, &x)| (i as f64, x as f64)
            } else {
                |(i, &x)| (x as f64, i as f64)
            })
            .collect::<Vec<(f64, f64)>>();

        let dataset = Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .data(&luma_histogram);

        let lum_axis = Axis::default()
            .style(Style::default().dark_gray())
            .bounds([0.0, 255.0])
            .labels(["0(B)", "128", "255(W)"]);

        let pixel_axis = Axis::default()
            .title("Pixel Count")
            .style(Style::default().dark_gray())
            .bounds([0.0, max_count])
            .labels(if area.width < Self::VECTORSCOPE_WIDTH {
                vec![Span::raw("0"), Span::raw(format!("{}", max_count as u64))]
            } else {
                vec![
                    Span::raw("0"),
                    Span::raw(format!("{}", (max_count / 2.0) as u64)),
                    Span::raw(format!("{}", max_count as u64)),
                ]
            });

        Block::bordered()
            .title("Luma Histogram")
            .border_type(Rounded)
            .render(scopes_layout[0], buf);
        Chart::new(vec![dataset])
            .x_axis(if self.app_layout == AppLayout::Vertical {
                lum_axis.clone()
            } else {
                pixel_axis.clone()
            })
            .y_axis(if self.app_layout == AppLayout::Vertical {
                pixel_axis
            } else {
                lum_axis
            })
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
