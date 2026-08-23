use crate::app::SliderData;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Text,
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
            Style::default().fg(Color::Gray)
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
            .split(if area.height > 15 {
                Rect { height: 15, ..area }
            } else {
                area
            });

        for (index, slider) in self.sliders.iter().enumerate() {
            let is_selected = index == self.selected_index;

            Block::default()
                .title(slider.label)
                .borders(Borders::ALL)
                .border_type(Rounded)
                .border_style(slider.style(is_selected))
                .render(slider_layout[index], buf);

            let slider_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(slider_layout[index].inner(Margin::new(0, 2)));
            Slider::from_state(&slider.state)
                .orientation(SliderOrientation::Vertical)
                .filled_symbol(slider.slider_style.filled_symbol)
                .handle_symbol(slider.slider_style.handle_symbol)
                .empty_symbol(slider.slider_style.empty_symbol)
                .filled_color(slider.color(is_selected))
                .handle_color(slider.color(is_selected))
                .empty_color(Color::DarkGray)
                .render(slider_layout[0], buf);
            Text::from(format!("{:.2}\n", slider.state.value()))
                .style(slider.style(is_selected))
                .alignment(Alignment::Center)
                .render(slider_layout[2], buf);
        }
    }
}
