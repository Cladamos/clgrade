use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Text};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Borders, Widget};

use crate::app::AppLayout;
use crate::ui::{CenterOpts, centered_rect, focused_style};

#[derive(Debug, Clone, Copy)]
pub enum ColorEffects {
    WhiteBalance,
    Exposure,
    Contrast,
    LiftGammaGain,
    Hue,
    Saturation,
}
impl ColorEffects {
    pub fn default() -> Vec<ColorEffects> {
        vec![
            ColorEffects::WhiteBalance,
            ColorEffects::Exposure,
            ColorEffects::Contrast,
            ColorEffects::LiftGammaGain,
            ColorEffects::Hue,
            ColorEffects::Saturation,
        ]
    }

    pub fn get_short_name(&self) -> &str {
        match self {
            ColorEffects::WhiteBalance => "WB",
            ColorEffects::Exposure => "EXP",
            ColorEffects::Contrast => "CONT",
            ColorEffects::LiftGammaGain => "WHEELS",
            ColorEffects::Hue => "HUE",
            ColorEffects::Saturation => "SAT",
        }
    }
}

pub struct PipelineSection<'a> {
    pipeline: &'a Vec<ColorEffects>,
    selected_index: usize,
    app_layout: AppLayout,
}

impl<'a> PipelineSection<'a> {
    // Horizontal layout
    pub const PIPE_HEIGHT: u16 = 1;
    pub const BOX_HEIGHT: u16 = 5;
    pub const PIPELINE_WIDTH: u16 = 21;
    // Vertical layout
    pub const PIPE_WIDTH: u16 = 2;
    pub const BOX_WIDTH: u16 = 10;
    pub const PIPELINE_HEIGHT: u16 = 5;

    pub fn new(
        pipeline: &'a Vec<ColorEffects>,
        selected_index: usize,
        app_layout: AppLayout,
    ) -> Self {
        PipelineSection {
            pipeline,
            selected_index,
            app_layout,
        }
    }

    pub fn col_height(pipeline: &Vec<ColorEffects>) -> u16 {
        // n boxes and n+1 pipes
        pipeline.len() as u16 * (Self::BOX_HEIGHT + Self::PIPE_HEIGHT) + Self::PIPE_HEIGHT
    }

    pub fn row_width(pipeline: &Vec<ColorEffects>) -> u16 {
        pipeline.len() as u16 * (Self::BOX_WIDTH + Self::PIPE_WIDTH) + Self::PIPE_WIDTH
    }
}

impl<'a> Widget for PipelineSection<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (
            widget_direction,
            pipeline_direction,
            pipe_length,
            box_length,
            pipeline_length,
            pipe_symbol,
            last_pipe_symbol,
        ) = match self.app_layout {
            AppLayout::Horizontal => (
                Direction::Horizontal,
                Direction::Vertical,
                Self::PIPE_HEIGHT,
                Self::BOX_HEIGHT,
                Self::PIPELINE_WIDTH,
                "|",
                "v",
            ),
            AppLayout::Vertical => (
                Direction::Vertical,
                Direction::Horizontal,
                Self::PIPE_WIDTH,
                Self::BOX_WIDTH,
                Self::PIPELINE_HEIGHT,
                "==",
                "=>",
            ),
        };

        let constraints = self
            .pipeline
            .iter()
            .flat_map(|_| {
                vec![
                    Constraint::Length(pipe_length),
                    Constraint::Length(box_length),
                ]
            })
            .chain(vec![Constraint::Length(pipe_length)]);

        let layout = Layout::default()
            .direction(widget_direction)
            .constraints(vec![Constraint::Length(pipeline_length)])
            .split(area);

        let pipeline_layout = Layout::default()
            .direction(pipeline_direction)
            .constraints(constraints)
            .split(layout[0]);

        let pipe_center_opts = CenterOpts {
            width: pipe_length,
            height: 1,
            margin: 0,
        };

        for (i, effect) in self.pipeline.iter().enumerate() {
            let is_focused = self.selected_index == i;
            let style = focused_style(is_focused);
            let layout_index = i * 2;
            Line::from(pipe_symbol).style(style).render(
                centered_rect(pipe_center_opts, pipeline_layout[layout_index]),
                buf,
            );
            Block::default()
                .borders(Borders::ALL)
                .border_type(Rounded)
                .style(style)
                .render(pipeline_layout[layout_index + 1], buf);

            let name = effect.get_short_name();
            Text::from(name).render(
                centered_rect(
                    super::CenterOpts {
                        width: name.len() as u16,
                        height: 1,
                        margin: 0,
                    },
                    pipeline_layout[layout_index + 1],
                ),
                buf,
            );

            if i == self.pipeline.len() - 1 {
                Line::from(last_pipe_symbol).style(style).render(
                    centered_rect(pipe_center_opts, pipeline_layout[layout_index + 2]),
                    buf,
                );
            }
        }
    }
}
