use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Widget},
};

use crate::{
    app::AppLayout,
    ui::{
        focused_style,
        slider::{SliderData, color_mixer_sliders},
    },
};
#[derive(Debug, Clone)]
pub struct ColorMixerPart {
    pub indicator_color: Color,
    pub sliders: [SliderData; 3],
}
impl ColorMixerPart {
    pub fn default_parts() -> Vec<ColorMixerPart> {
        let colors = vec![
            Color::Indexed(196),
            Color::Indexed(208),
            Color::Indexed(226),
            Color::Indexed(46),
            Color::Indexed(51),
            Color::Indexed(33),
            Color::Indexed(129),
            Color::Indexed(201),
        ];
        let mut parts = vec![];
        for c in colors {
            parts.push(ColorMixerPart {
                indicator_color: c,
                sliders: color_mixer_sliders(),
            });
        }
        parts
    }
}
pub struct ColorMixerSection<'a> {
    pub parts: &'a [ColorMixerPart],
    pub app_layout: AppLayout,
    pub selected_color: usize,
    pub is_color_section_selected: bool,
}
impl<'a> ColorMixerSection<'a> {
    pub fn new(
        parts: &'a [ColorMixerPart],
        app_layout: AppLayout,
        selected_color: usize,
        is_color_section_selected: bool,
    ) -> Self {
        Self {
            parts,
            app_layout,
            selected_color,
            is_color_section_selected,
        }
    }
}
impl<'a> Widget for ColorMixerSection<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (
            border_area,
            border_direction,
            border_margin,
            border_constraints,
            inner_constraints,
            inner_direction,
        ) = match self.app_layout {
            AppLayout::Horizontal => (
                Rect {
                    x: area.x,
                    y: area.y,
                    width: 21,
                    height: 4,
                },
                Direction::Vertical,
                Margin::new(3, 1),
                vec![Constraint::Length(1), Constraint::Length(1)],
                self.parts
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        // Don't add gap to last part for easier centering
                        if i == self.parts.len() - 1 {
                            Constraint::Length(1)
                        } else {
                            Constraint::Length(2)
                        }
                    })
                    .collect::<Vec<Constraint>>(),
                Direction::Horizontal,
            ),
            AppLayout::Vertical => (
                Rect {
                    x: area.x,
                    y: area.y,
                    width: 8,
                    height: 10,
                },
                Direction::Horizontal,
                Margin::new(2, 1),
                vec![Constraint::Length(2), Constraint::Length(1)],
                self.parts
                    .iter()
                    .enumerate()
                    .map(|_| Constraint::Length(1))
                    .collect::<Vec<Constraint>>(),
                Direction::Vertical,
            ),
        };
        let select_indicator = match self.app_layout {
            AppLayout::Horizontal => "^",
            AppLayout::Vertical => "<",
        };
        let border_layout = Layout::default()
            .direction(border_direction)
            .constraints(border_constraints)
            .split(border_area.inner(border_margin));
        let indicator_layout = Layout::default()
            .direction(inner_direction)
            .constraints(inner_constraints.clone())
            .split(border_layout[1]);
        let colors_layout = Layout::default()
            .direction(inner_direction)
            .constraints(inner_constraints)
            .split(border_layout[0]);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(focused_style(self.is_color_section_selected))
            .render(border_area, buf);
        for (index, color) in self.parts.iter().enumerate() {
            Span::styled("●", Style::default().fg(color.indicator_color))
                .render(colors_layout[index], buf);
            if self.selected_color == index {
                Span::styled(select_indicator, Style::default().fg(color.indicator_color))
                    .render(indicator_layout[index], buf);
            }
        }
    }
}
