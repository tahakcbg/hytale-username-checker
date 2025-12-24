mod app;
mod checker;
mod proxy;
mod ui;

use iced::Theme;

fn main() -> iced::Result {
    iced::application(app::App::new, app::App::update, app::App::view)
        .title("Hytale Username Checker")
        .theme(theme)
        .window_size((960.0, 600.0))
        .centered()
        .run()
}

fn theme(_state: &app::App) -> Theme {
    Theme::custom(
        "HytaleChecker".to_string(),
        iced::theme::Palette {
            background: ui::theme::BG_DEEP,
            text: ui::theme::TEXT_PRIMARY,
            primary: ui::theme::ACCENT_PURPLE,
            success: ui::theme::SUCCESS,
            warning: ui::theme::WARNING,
            danger: ui::theme::DANGER,
        },
    )
}
