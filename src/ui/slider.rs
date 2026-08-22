use crate::app::SliderData;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType::Rounded, Borders, Widget},
};
use tui_slider::{Slider, SliderOrientation};

impl SliderData {
    fn style(&self, selected: bool) -> Style {
        if selected {
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    }

    fn color(&self, selected: bool) -> Color {
        if selected { Color::Blue } else { Color::Gray }
    }
}

pub struct SliderSection<'a> {
    sliders: &'a [SliderData],
    selected_index: usize,
}

impl<'a> SliderSection<'a> {
    pub fn new(sliders: &'a [SliderData], selected_index: usize) -> Self {
        Self {
            sliders,
            selected_index,
        }
    }
}

impl<'a> Widget for SliderSection<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let slider_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(self.sliders.iter().map(|_| Constraint::Length(15)))
            .split(area);

        for (index, slider) in self.sliders.iter().enumerate() {
            let is_selected = index == self.selected_index;

            // Render border box
            Block::default()
                .title(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(slider.label, slider.style(is_selected)),
                ]))
                .borders(Borders::ALL)
                .border_type(Rounded)
                .border_style(slider.style(is_selected))
                .render(slider_layout[index], buf);

            // Render inner vertical slider

            // TODO: make it responsive for terminal resize.
            // TODO: add horizontal split option
            Slider::from_state(&slider.state)
                .orientation(SliderOrientation::Vertical)
                .filled_symbol(slider.slider_style.filled_symbol)
                .handle_symbol(slider.slider_style.handle_symbol)
                .empty_symbol(slider.slider_style.empty_symbol)
                .filled_color(slider.color(is_selected))
                .handle_color(slider.color(is_selected))
                .empty_color(Color::DarkGray)
                .show_value(true)
                .render(slider_layout[index].inner(Margin::new(1, 2)), buf);
        }
    }
}
