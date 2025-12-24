use iced::widget::{button, container, row, space, text, text_editor, text_input};
use iced::{border::Radius, Alignment, Background, Border, Color, Element, Fill};

use super::theme;
use crate::app::Message;

pub fn glass_card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(container(content).width(Fill).height(Fill).padding(18))
        .width(Fill)
        .height(Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::BG_CARD)),
            border: Border {
                color: theme::BORDER_SUBTLE,
                width: 1.0,
                radius: Radius::new(14),
            },
            ..Default::default()
        })
        .into()
}

pub fn stat_pill<'a>(icon: &'a str, value: usize, color: Color) -> Element<'a, Message> {
    container(
        row![
            text(icon).size(11).color(color),
            space::horizontal().width(5),
            text(value.to_string())
                .size(12)
                .color(theme::TEXT_PRIMARY),
        ]
        .align_y(Alignment::Center),
    )
    .padding([5, 12])
    .style(move |_| container::Style {
        background: Some(Background::Color(color.scale_alpha(0.12))),
        border: Border {
            color: color.scale_alpha(0.25),
            width: 1.0,
            radius: Radius::new(20),
        },
        ..Default::default()
    })
    .into()
}

pub fn action_button<'a>(
    label: &'a str,
    color: Color,
    enabled: bool,
) -> button::Button<'a, Message> {
    button(
        text(label).size(12).color(if enabled {
            theme::TEXT_BRIGHT
        } else {
            theme::TEXT_MUTED
        }),
    )
    .padding([10, 20])
    .style(move |_, status| {
        let bg = if !enabled {
            theme::BG_INPUT
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => color.scale_alpha(0.85),
                _ => color,
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if enabled {
                theme::TEXT_BRIGHT
            } else {
                theme::TEXT_MUTED
            },
            border: Border {
                radius: Radius::new(8),
                ..Default::default()
            },
            ..Default::default()
        }
    })
}

pub fn secondary_button<'a>(label: &'a str, enabled: bool) -> button::Button<'a, Message> {
    button(
        text(label).size(12).color(if enabled {
            theme::TEXT_SECONDARY
        } else {
            theme::TEXT_MUTED
        }),
    )
    .padding([10, 18])
    .style(move |_, status| {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed if enabled => theme::BG_ELEVATED,
            _ => theme::BG_INPUT,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if enabled {
                theme::TEXT_SECONDARY
            } else {
                theme::TEXT_MUTED
            },
            border: Border {
                color: theme::BORDER_SUBTLE,
                width: 1.0,
                radius: Radius::new(8),
            },
            ..Default::default()
        }
    })
}

pub fn editor_style() -> text_editor::Style {
    text_editor::Style {
        background: Background::Color(theme::BG_INPUT),
        border: Border {
            color: theme::BORDER_SUBTLE,
            width: 1.0,
            radius: Radius::new(10),
        },
        placeholder: theme::TEXT_MUTED,
        value: theme::TEXT_PRIMARY,
        selection: theme::ACCENT_PURPLE.scale_alpha(0.3),
    }
}

pub fn input_style() -> text_input::Style {
    text_input::Style {
        background: Background::Color(theme::BG_INPUT),
        border: Border {
            color: theme::BORDER_SUBTLE,
            width: 1.0,
            radius: Radius::new(6),
        },
        icon: theme::TEXT_MUTED,
        placeholder: theme::TEXT_MUTED,
        value: theme::TEXT_PRIMARY,
        selection: theme::ACCENT_CYAN.scale_alpha(0.3),
    }
}
