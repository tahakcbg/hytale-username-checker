use chrono::Local;
use iced::widget::{
    button, column, container, pick_list, row, scrollable, space, text, text_editor, text_input,
    Column,
};
use iced::{border::Radius, Alignment, Background, Border, Color, Element, Fill, Length, Task};

use crate::checker::{check_usernames_stream, CheckEvent, CheckResult, ResultStatus, Stats};
use crate::proxy::ProxyType;
use crate::ui::{self, theme};

#[derive(Debug, Clone)]
pub enum Message {
    UsernamesChanged(text_editor::Action),
    ProxiesChanged(text_editor::Action),
    ProxyTypeChanged(ProxyType),
    DelayChanged(String),
    ConcurrencyChanged(String),
    TabChanged(Tab),
    ToggleProxyPanel,
    StartCheck,
    StopCheck,
    CheckEventReceived(CheckEvent),
    ExportResults,
    ExportComplete(Result<String, String>),
    ClearResults,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    All,
    Available,
    Taken,
    Errors,
}

pub struct App {
    usernames_content: text_editor::Content,
    proxies_content: text_editor::Content,
    proxy_type: ProxyType,
    delay_ms: String,
    concurrency: String,
    current_tab: Tab,
    is_checking: bool,
    results: Vec<CheckResult>,
    stats: Stats,
    status_message: String,
    show_proxy_panel: bool,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                usernames_content: text_editor::Content::new(),
                proxies_content: text_editor::Content::new(),
                proxy_type: ProxyType::None,
                delay_ms: "100".to_string(),
                concurrency: "5".to_string(),
                current_tab: Tab::All,
                is_checking: false,
                results: Vec::new(),
                stats: Stats::default(),
                status_message: String::new(),
                show_proxy_panel: false,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UsernamesChanged(action) => {
                self.usernames_content.perform(action);
                Task::none()
            }
            Message::ProxiesChanged(action) => {
                self.proxies_content.perform(action);
                Task::none()
            }
            Message::ProxyTypeChanged(proxy_type) => {
                self.proxy_type = proxy_type;
                Task::none()
            }
            Message::DelayChanged(value) => {
                if value.is_empty() || value.parse::<u64>().is_ok() {
                    self.delay_ms = value;
                }
                Task::none()
            }
            Message::ConcurrencyChanged(value) => {
                if value.is_empty() || value.parse::<usize>().is_ok() {
                    self.concurrency = value;
                }
                Task::none()
            }
            Message::TabChanged(tab) => {
                self.current_tab = tab;
                Task::none()
            }
            Message::ToggleProxyPanel => {
                self.show_proxy_panel = !self.show_proxy_panel;
                Task::none()
            }
            Message::StartCheck => {
                let usernames: Vec<String> = self
                    .usernames_content
                    .text()
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if usernames.is_empty() {
                    self.status_message = "Enter usernames to check".to_string();
                    return Task::none();
                }

                self.is_checking = true;
                self.results.clear();
                self.stats = Stats {
                    total: usernames.len(),
                    ..Default::default()
                };
                self.status_message = format!("Checking {} usernames...", usernames.len());

                let delay = self.delay_ms.parse().unwrap_or(100);
                let concurrency = self.concurrency.parse().unwrap_or(5).max(1);

                let proxies: Vec<String> = if self.proxy_type != ProxyType::None {
                    self.proxies_content
                        .text()
                        .lines()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .map(|s| self.proxy_type.format_proxy(&s))
                        .collect()
                } else {
                    Vec::new()
                };

                let rx = check_usernames_stream(usernames, proxies, delay, concurrency);
                Task::run(rx, Message::CheckEventReceived)
            }
            Message::StopCheck => {
                self.is_checking = false;
                self.status_message = "Stopped".to_string();
                Task::none()
            }
            Message::CheckEventReceived(event) => {
                match event {
                    CheckEvent::Result(result) => {
                        match &result.status {
                            ResultStatus::Available => self.stats.available += 1,
                            ResultStatus::Taken => self.stats.taken += 1,
                            ResultStatus::Error(_) | ResultStatus::Invalid => self.stats.errors += 1,
                        }
                        self.stats.checked += 1;
                        self.results.push(result);
                    }
                    CheckEvent::Done => {
                        self.is_checking = false;
                        self.status_message = "Complete".to_string();
                    }
                }
                Task::none()
            }
            Message::ExportResults => {
                let available: Vec<String> = self
                    .results
                    .iter()
                    .filter(|r| r.status == ResultStatus::Available)
                    .map(|r| r.username.clone())
                    .collect();

                if available.is_empty() {
                    self.status_message = "No available usernames".to_string();
                    return Task::none();
                }

                Task::perform(export_to_file(available), Message::ExportComplete)
            }
            Message::ExportComplete(result) => {
                self.status_message = match result {
                    Ok(path) => format!("Saved: {}", path),
                    Err(e) => e,
                };
                Task::none()
            }
            Message::ClearResults => {
                self.results.clear();
                self.stats = Stats::default();
                self.status_message.clear();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let header = self.view_header();
        let main_content = self.view_main();
        let footer = self.view_footer();

        let layout = column![
            header,
            space::vertical().height(16),
            main_content,
            space::vertical().height(12),
            footer,
        ]
        .width(Fill)
        .height(Fill);

        container(layout)
            .style(|_| container::Style {
                background: Some(Background::Color(theme::BG_DEEP)),
                ..Default::default()
            })
            .width(Fill)
            .height(Fill)
            .padding(24)
            .into()
    }

    fn view_header(&self) -> Element<'_, Message> {
        let title_area = column![
            text("HYTALE").size(16).color(theme::TEXT_BRIGHT),
            text("Username Checker")
                .size(11)
                .color(theme::TEXT_SECONDARY),
        ]
        .spacing(1);

        let stats_pills = row![
            ui::stat_pill("✓", self.stats.available, theme::SUCCESS),
            ui::stat_pill("✗", self.stats.taken, theme::DANGER),
            ui::stat_pill("!", self.stats.errors, theme::WARNING),
        ]
        .spacing(8);

        row![
            space::horizontal().width(14),
            title_area,
            space::horizontal().width(Fill),
            stats_pills,
        ]
        .align_y(Alignment::Center)
        .into()
    }

    fn view_main(&self) -> Element<'_, Message> {
        let left_panel = self.view_input_panel();
        let right_panel = self.view_results_panel();

        row![
            container(left_panel).width(Length::FillPortion(5)),
            space::horizontal().width(20),
            container(right_panel).width(Length::FillPortion(6)),
        ]
        .height(Fill)
        .into()
    }

    fn view_input_panel(&self) -> Element<'_, Message> {
        let username_header = row![
            text("Usernames").size(12).color(theme::TEXT_PRIMARY),
            space::horizontal().width(Fill),
            text("one per line").size(10).color(theme::TEXT_MUTED),
        ];

        let username_editor = text_editor(&self.usernames_content)
            .placeholder("dream\nnotch\njeb_\n...")
            .on_action(Message::UsernamesChanged)
            .padding(14)
            .height(if self.show_proxy_panel {
                Length::Fixed(140.0)
            } else {
                Fill
            })
            .style(|_, _| ui::editor_style());

        let mut content = column![
            username_header,
            space::vertical().height(10),
            username_editor,
        ];

        let proxy_toggle = button(
            row![
                text(if self.show_proxy_panel { "▼" } else { "▶" })
                    .size(10)
                    .color(theme::ACCENT_CYAN),
                space::horizontal().width(8),
                text("Proxy Settings")
                    .size(11)
                    .color(theme::TEXT_SECONDARY),
                space::horizontal().width(Fill),
                text(if self.proxy_type == ProxyType::None {
                    "disabled"
                } else {
                    "enabled"
                })
                .size(10)
                .color(if self.proxy_type == ProxyType::None {
                    theme::TEXT_MUTED
                } else {
                    theme::SUCCESS
                }),
            ]
            .align_y(Alignment::Center)
            .width(Fill),
        )
        .width(Fill)
        .padding([10, 14])
        .style(|_, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => theme::BG_ELEVATED,
                _ => theme::BG_INPUT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: theme::TEXT_SECONDARY,
                border: Border {
                    color: theme::BORDER_SUBTLE,
                    width: 1.0,
                    radius: Radius::new(8),
                },
                ..Default::default()
            }
        })
        .on_press(Message::ToggleProxyPanel);

        content = content
            .push(space::vertical().height(12))
            .push(proxy_toggle);

        if self.show_proxy_panel {
            let proxy_section = self.view_proxy_section();
            content = content
                .push(space::vertical().height(12))
                .push(proxy_section);
        }

        let settings_row = self.view_settings_row();
        content = content
            .push(space::vertical().height(12))
            .push(settings_row);

        ui::glass_card(content)
    }

    fn view_proxy_section(&self) -> Element<'_, Message> {
        let type_picker = pick_list(
            ProxyType::ALL.as_slice(),
            Some(self.proxy_type),
            Message::ProxyTypeChanged,
        )
        .placeholder("Select type...")
        .padding([8, 12])
        .text_size(12)
        .style(|_, _| pick_list::Style {
            text_color: theme::TEXT_PRIMARY,
            placeholder_color: theme::TEXT_MUTED,
            handle_color: theme::ACCENT_CYAN,
            background: Background::Color(theme::BG_INPUT),
            border: Border {
                color: theme::BORDER_SUBTLE,
                width: 1.0,
                radius: Radius::new(6),
            },
        });

        let type_row = row![
            text("Type").size(11).color(theme::TEXT_MUTED),
            space::horizontal().width(12),
            type_picker,
        ]
        .align_y(Alignment::Center);

        let proxy_editor = text_editor(&self.proxies_content)
            .placeholder("host:port\nuser:pass@host:port\n...")
            .on_action(Message::ProxiesChanged)
            .padding(12)
            .height(100)
            .style(|_, _| ui::editor_style());

        let hint = text("Proxies rotate automatically. Format: host:port or user:pass@host:port")
            .size(10)
            .color(theme::TEXT_MUTED);

        container(column![
            type_row,
            space::vertical().height(10),
            proxy_editor,
            space::vertical().height(6),
            hint,
        ])
        .width(Fill)
        .padding(14)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::BG_INPUT)),
            border: Border {
                color: theme::BORDER_ACCENT,
                width: 1.0,
                radius: Radius::new(8),
            },
            ..Default::default()
        })
        .into()
    }

    fn view_settings_row(&self) -> Element<'_, Message> {
        let delay_input = text_input("100", &self.delay_ms)
            .on_input(Message::DelayChanged)
            .padding([8, 10])
            .size(12)
            .width(65)
            .style(|_, _| ui::input_style());

        let threads_input = text_input("5", &self.concurrency)
            .on_input(Message::ConcurrencyChanged)
            .padding([8, 10])
            .size(12)
            .width(50)
            .style(|_, _| ui::input_style());

        row![
            text("Delay").size(11).color(theme::TEXT_MUTED),
            space::horizontal().width(6),
            delay_input,
            text("ms").size(10).color(theme::TEXT_MUTED),
            space::horizontal().width(Fill),
            text("Threads").size(11).color(theme::TEXT_MUTED),
            space::horizontal().width(6),
            threads_input,
        ]
        .align_y(Alignment::Center)
        .into()
    }

    fn view_results_panel(&self) -> Element<'_, Message> {
        let tabs = row![
            glow_tab("All", Tab::All, self.current_tab, self.results.len()),
            glow_tab(
                "Available",
                Tab::Available,
                self.current_tab,
                self.stats.available
            ),
            glow_tab("Taken", Tab::Taken, self.current_tab, self.stats.taken),
            glow_tab("Errors", Tab::Errors, self.current_tab, self.stats.errors),
        ]
        .spacing(6);

        let progress = if self.stats.total > 0 {
            self.stats.checked as f32 / self.stats.total as f32
        } else {
            0.0
        };

        let progress_text = text(format!("{}/{}", self.stats.checked, self.stats.total))
            .size(11)
            .color(theme::TEXT_MUTED);

        let header_row =
            row![tabs, space::horizontal().width(Fill), progress_text].align_y(Alignment::Center);

        let progress_bar = self.view_progress_bar(progress);

        let filtered: Vec<_> = self
            .results
            .iter()
            .filter(|r| match self.current_tab {
                Tab::All => true,
                Tab::Available => r.status == ResultStatus::Available,
                Tab::Taken => r.status == ResultStatus::Taken,
                Tab::Errors => matches!(r.status, ResultStatus::Error(_) | ResultStatus::Invalid),
            })
            .collect();

        let results_content: Element<'_, Message> = if filtered.is_empty() {
            container(
                column![
                    text(if self.is_checking {
                        "◌"
                    } else if self.results.is_empty() {
                        "○"
                    } else {
                        "∅"
                    })
                    .size(32)
                    .color(theme::TEXT_MUTED),
                    space::vertical().height(8),
                    text(if self.is_checking {
                        "Checking..."
                    } else if self.results.is_empty() {
                        "No results yet"
                    } else {
                        "No matches in category"
                    })
                    .size(12)
                    .color(theme::TEXT_MUTED),
                ]
                .align_x(Alignment::Center),
            )
            .width(Fill)
            .height(Fill)
            .center(Fill)
            .into()
        } else {
            let items: Vec<Element<'_, Message>> =
                filtered.iter().map(|r| result_row(r)).collect();

            scrollable(Column::with_children(items).spacing(4).padding(8))
                .height(Fill)
                .into()
        };

        let results_box = container(results_content)
            .width(Fill)
            .height(Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(theme::BG_INPUT)),
                border: Border {
                    radius: Radius::new(10),
                    ..Default::default()
                },
                ..Default::default()
            });

        ui::glass_card(column![
            header_row,
            space::vertical().height(12),
            progress_bar,
            space::vertical().height(12),
            results_box,
        ])
    }

    fn view_progress_bar(&self, progress: f32) -> Element<'_, Message> {
        let bar_width = (progress * 100.0).max(0.0).min(100.0);

        let filled = container(space::horizontal())
            .width(Length::FillPortion(bar_width as u16))
            .height(4)
            .style(|_| container::Style {
                background: Some(Background::Color(theme::ACCENT_CYAN)),
                border: Border {
                    radius: Radius::new(2),
                    ..Default::default()
                },
                ..Default::default()
            });

        let empty = container(space::horizontal())
            .width(Length::FillPortion((100.0 - bar_width) as u16))
            .height(4);

        container(row![filled, empty])
            .width(Fill)
            .height(4)
            .style(|_| container::Style {
                background: Some(Background::Color(theme::BG_DEEP)),
                border: Border {
                    radius: Radius::new(2),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    fn view_footer(&self) -> Element<'_, Message> {
        let start_btn = ui::action_button(
            if self.is_checking {
                "Checking..."
            } else {
                "Start Check"
            },
            theme::ACCENT_PURPLE,
            !self.is_checking,
        )
        .on_press_maybe(if !self.is_checking {
            Some(Message::StartCheck)
        } else {
            None
        });

        let stop_btn =
            ui::action_button("Stop", theme::DANGER, self.is_checking).on_press_maybe(
                if self.is_checking {
                    Some(Message::StopCheck)
                } else {
                    None
                },
            );

        let export_btn = ui::action_button(
            "Export",
            theme::ACCENT_BLUE,
            self.stats.available > 0 && !self.is_checking,
        )
        .on_press_maybe(if self.stats.available > 0 && !self.is_checking {
            Some(Message::ExportResults)
        } else {
            None
        });

        let clear_btn =
            ui::secondary_button("Clear", !self.results.is_empty() && !self.is_checking)
                .on_press_maybe(if !self.results.is_empty() && !self.is_checking {
                    Some(Message::ClearResults)
                } else {
                    None
                });

        let status = text(&self.status_message)
            .size(11)
            .color(theme::TEXT_SECONDARY);

        row![
            start_btn,
            stop_btn,
            space::horizontal().width(20),
            status,
            space::horizontal().width(Fill),
            export_btn,
            clear_btn,
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
    }
}

fn glow_tab<'a>(label: &'a str, tab: Tab, current: Tab, count: usize) -> Element<'a, Message> {
    let is_active = tab == current;
    let accent = match tab {
        Tab::All => theme::ACCENT_PURPLE,
        Tab::Available => theme::SUCCESS,
        Tab::Taken => theme::DANGER,
        Tab::Errors => theme::WARNING,
    };

    button(
        row![
            text(label).size(11).color(if is_active {
                theme::TEXT_BRIGHT
            } else {
                theme::TEXT_MUTED
            }),
            space::horizontal().width(4),
            text(count.to_string()).size(9).color(if is_active {
                accent
            } else {
                theme::TEXT_MUTED
            }),
        ]
        .align_y(Alignment::Center),
    )
    .padding([6, 12])
    .style(move |_, status| {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => {
                if is_active {
                    accent.scale_alpha(0.25)
                } else {
                    theme::BG_ELEVATED
                }
            }
            _ if is_active => accent.scale_alpha(0.18),
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if is_active {
                theme::TEXT_BRIGHT
            } else {
                theme::TEXT_MUTED
            },
            border: Border {
                color: if is_active {
                    accent.scale_alpha(0.4)
                } else {
                    Color::TRANSPARENT
                },
                width: 1.0,
                radius: Radius::new(8),
            },
            ..Default::default()
        }
    })
    .on_press(Message::TabChanged(tab))
    .into()
}

fn result_row<'a>(result: &'a CheckResult) -> Element<'a, Message> {
    let (icon, color) = match &result.status {
        ResultStatus::Available => ("●", theme::SUCCESS),
        ResultStatus::Taken => ("○", theme::DANGER),
        ResultStatus::Error(_) | ResultStatus::Invalid => ("◌", theme::WARNING),
    };

    container(
        row![
            container(text(icon).size(8).color(color))
                .width(20)
                .center(20),
            text(&result.username)
                .size(12)
                .color(theme::TEXT_PRIMARY),
            space::horizontal().width(Fill),
            text(match &result.status {
                ResultStatus::Available => "available",
                ResultStatus::Taken => "taken",
                ResultStatus::Error(e) => e.as_str(),
                ResultStatus::Invalid => "invalid",
            })
            .size(10)
            .color(color),
        ]
        .align_y(Alignment::Center),
    )
    .padding([8, 12])
    .style(move |_| container::Style {
        background: Some(Background::Color(theme::BG_CARD)),
        border: Border {
            color: color.scale_alpha(0.15),
            width: 1.0,
            radius: Radius::new(8),
        },
        ..Default::default()
    })
    .into()
}

async fn export_to_file(usernames: Vec<String>) -> Result<String, String> {
    let dialog = rfd::AsyncFileDialog::new()
        .add_filter("Text files", &["txt"])
        .set_file_name(format!(
            "hytale_available_{}.txt",
            Local::now().format("%Y%m%d_%H%M%S")
        ))
        .save_file()
        .await;

    match dialog {
        Some(handle) => {
            let content = usernames.join("\n");
            handle
                .write(content.as_bytes())
                .await
                .map(|_| handle.path().to_string_lossy().to_string())
                .map_err(|e| e.to_string())
        }
        None => Err("Cancelled".into()),
    }
}
