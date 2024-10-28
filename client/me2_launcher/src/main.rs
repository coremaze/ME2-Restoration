#![windows_subsystem = "windows"]
use config::{Config, File};
use iced::widget::text_input;
use iced::{
    executor,
    widget::{button, Button, Column, Container, Space, Text, TextInput},
    Application, Background, BorderRadius, Color, Command, Element, Length, Settings,
};

use std::process::Command as StdCommand;

struct Launcher {
    ip: String,
    session: String,
}

#[derive(Debug, Clone)]
enum Message {
    IpChanged(String),
    SessionChanged(String),
    LoginPressed,
}

struct CustomButtonStyle;

impl button::StyleSheet for CustomButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(iced::Color::BLACK)),
            border_radius: BorderRadius::from(6.0),
            text_color: iced::Color::WHITE,
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(iced::Color::BLACK)),
            border_radius: BorderRadius::from(6.0),
            text_color: iced::Color::WHITE,
            ..button::Appearance::default()
        }
    }
}
struct CustomInputStyle;

impl text_input::StyleSheet for CustomInputStyle {
    type Style = iced::Theme;

    fn active(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::WHITE),
            border_radius: BorderRadius::from(6.0),
            border_width: 1.0,
            border_color: Color::BLACK,
            icon_color: Color::BLACK,
        }
    }

    fn focused(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            border_color: iced::Color::from_rgb(0.0, 0.5, 1.0),
            ..self.active(&iced::Theme::default())
        }
    }

    fn placeholder_color(&self, _: &Self::Style) -> iced::Color {
        iced::Color::from_rgb(0.5, 0.5, 0.5)
    }

    fn value_color(&self, _: &Self::Style) -> iced::Color {
        iced::Color::BLACK
    }

    fn selection_color(&self, _: &Self::Style) -> iced::Color {
        iced::Color::from_rgb(0.8, 0.8, 0.8)
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        Color::from_rgb(0.5, 0.5, 0.5)
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.5, 0.5, 0.5)),
            ..self.active(style)
        }
    }
}

impl Application for Launcher {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = iced::Theme;

    fn new(_flags: ()) -> (Launcher, Command<Message>) {
        let settings = Config::builder()
            .add_source(File::with_name("launch_config"))
            .build()
            .unwrap_or_else(|_| {
                // Create default launch_config.ini if it doesn't exist
                std::fs::write(
                    "launch_config.ini",
                    "[Settings]\nip=127.0.0.1\nsession=MyName",
                )
                .expect("Failed to create launch_config.ini");

                // Reload the settings after creating the file
                Config::builder()
                    .add_source(File::with_name("launch_config"))
                    .build()
                    .expect("Failed to load launch_config.ini after creation")
            });

        let ip = settings
            .get::<String>("Settings.ip")
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        let session = settings
            .get::<String>("Settings.session")
            .unwrap_or_else(|_| "MyName".to_string());

        (Launcher { ip, session }, Command::none())
    }

    fn title(&self) -> String {
        String::from("ME2 Launcher")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IpChanged(ip) => {
                self.ip = ip;
                Command::none()
            }
            Message::SessionChanged(session) => {
                self.session = session;
                Command::none()
            }
            Message::LoginPressed => {
                // Save to ini using `ConfigBuilder`
                std::fs::write(
                    "launch_config.ini",
                    format!("[Settings]\nip={}\nsession={}\n", self.ip, self.session),
                )
                .expect("Failed to write config");

                // Run game.exe
                let current_exe =
                    std::env::current_exe().expect("Failed to get current executable path");

                let current_dir = current_exe
                    .parent()
                    .expect("Failed to get parent directory");

                let injector_exe_path = current_dir.join("injector.exe");
                let game_exe_path = current_dir
                    .join("projector")
                    .join("PJ1159")
                    .join("Projector.exe");
                let dll_path = current_dir.join("me2hook.dll");
                let game_dcr_path = current_dir
                    .join("me2")
                    .join("iToys")
                    .join("ME2Data")
                    .join("me2Game.dcr");

                println!("{}", injector_exe_path.display());
                StdCommand::new(injector_exe_path)
                    .arg(game_exe_path)
                    .arg(dll_path)
                    .arg(game_dcr_path)
                    .spawn()
                    .expect("Failed to run game");

                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = Column::new()
            .padding(20)
            .spacing(10)
            .push(Text::new("IP Address:"))
            .push(
                TextInput::new("", &self.ip)
                    .on_input(Message::IpChanged)
                    .style(iced::theme::TextInput::Custom(Box::new(CustomInputStyle)))
                    .padding(10),
            )
            .push(Text::new("Display Name:"))
            .push(
                TextInput::new("", &self.session)
                    .on_input(Message::SessionChanged)
                    .style(iced::theme::TextInput::Custom(Box::new(CustomInputStyle)))
                    .padding(10),
            )
            .push(Space::with_height(Length::Fixed(10.0)))
            .push(
                Button::new(
                    Container::new(Text::new("Login"))
                        .width(Length::Fill)
                        .center_x()
                        .center_y(),
                )
                .style(iced::theme::Button::Custom(Box::new(CustomButtonStyle)))
                .on_press(Message::LoginPressed)
                .width(Length::Fill),
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            // .center_y()
            .into()
    }
}

fn main() -> iced::Result {
    let settings = Settings {
        window: iced::window::Settings {
            size: (300, 275),
            resizable: false,
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    };

    Launcher::run(settings)
}
