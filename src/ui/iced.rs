use crate::{
    config::{write_config_file, ClientConfig},
    constants::{ICON_BYTES, WINDOW_TITLE},
    core::{
        api::{lookup_server, LookupData, LookupError},
        reqwest,
    },
    servers::start_all_servers,
    ui::show_error,
    update,
};
use iced::{
    executor,
    theme::Palette,
    widget::{
        button, checkbox, column, container, row, text, text_input, Button, Column, Row, Text,
        TextInput,
    },
    window::{self, icon},
    Application, Color, Command, Length, Settings, Theme,
};

/// The window size
pub const WINDOW_SIZE: (u32, u32) = (500, 200);

pub fn init(config: Option<ClientConfig>, client: reqwest::Client) {
    App::run(Settings {
        window: window::Settings {
            icon: icon::from_file_data(ICON_BYTES, None).ok(),
            size: WINDOW_SIZE,
            resizable: false,

            ..window::Settings::default()
        },
        flags: (config, client),
        ..Settings::default()
    })
    .unwrap();
}

struct App {
    lookup_result: LookupState,
    remember: bool,
    target: String,
    http_client: reqwest::Client,
}

/// Messages used for updating the game state
#[derive(Debug, Clone)]
enum AppMessage {
    /// The redirector target address changed
    TargetChanged(String),
    /// The redirector target should be updated
    UpdateTarget,
    /// Message for setting the current lookup result state
    LookupState(LookupState),
    /// The remember checkbox button has changed
    RememberChanged(bool),
}

/// Different states that lookup process can be in
#[derive(Debug, Clone)]
enum LookupState {
    /// Lookup not yet done
    None,
    /// Looking up value
    Loading,
    /// Lookup complete success
    Success(LookupData),
    /// Lookup failed error
    Error,
}

impl Application for App {
    type Message = AppMessage;
    type Executor = executor::Default;
    type Flags = (Option<ClientConfig>, reqwest::Client);
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (config, http_client) = flags;
        let (target, remember) = config
            .map(|value| (value.connection_url, true))
            .unwrap_or_default();

        // Spawn the update checking task
        tokio::spawn(update::update(http_client.clone()));

        (
            App {
                lookup_result: LookupState::None,
                target,
                remember,
                http_client,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        WINDOW_TITLE.to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            // Update the stored target
            AppMessage::TargetChanged(value) => self.target = value,
            // Handle new target being set
            AppMessage::UpdateTarget => {
                // Don't try to lookup if already looking up
                if let LookupState::Loading = self.lookup_result {
                    return Command::none();
                }

                self.lookup_result = LookupState::Loading;

                let target = self.target.clone();

                // Handling for once the async lookup is complete
                let post_lookup = |result: Result<LookupData, LookupError>| {
                    let result = match result {
                        Ok(value) => LookupState::Success(value),
                        Err(err) => {
                            show_error("Failed to connect", &err.to_string());
                            LookupState::Error
                        }
                    };
                    AppMessage::LookupState(result)
                };

                // Perform the async lookup with the callback
                return Command::perform(
                    lookup_server(self.http_client.clone(), target),
                    post_lookup,
                );
            }

            // Lookup result changed
            AppMessage::LookupState(value) => {
                if let LookupState::Success(value) = &value {
                    // Start all the servers
                    start_all_servers(self.http_client.clone(), value.url.clone());

                    // Save the connection URL
                    if self.remember {
                        let connection_url = value.url.to_string();

                        write_config_file(ClientConfig { connection_url });
                    }
                }

                self.lookup_result = value;
            }

            // Remember value changed
            AppMessage::RememberChanged(value) => self.remember = value,
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        const DARK_TEXT: Color = Color::from_rgb(0.4, 0.4, 0.4);
        const RED_TEXT: Color = Color::from_rgb(0.8, 0.4, 0.4);
        const YELLOW_TEXT: Color = Color::from_rgb(0.8, 0.8, 0.4);
        const ORANGE_TEXT: Color = Color::from_rgb(0.8, 0.6, 0.4);
        const SPACING: u16 = 10;

        let target_input: TextInput<_> = text_input("Connection URL", &self.target)
            .padding(10)
            .on_input(AppMessage::TargetChanged)
            .on_submit(AppMessage::UpdateTarget);

        let target_text: Text =
            text("Please put the server Connection URL below and press 'Set'").style(DARK_TEXT);
        let target_button: Button<_> = button("Set").on_press(AppMessage::UpdateTarget).padding(10);

        let status_text: Text = match &self.lookup_result {
            LookupState::None => text("Not Connected.").style(ORANGE_TEXT),
            LookupState::Loading => text("Connecting...").style(YELLOW_TEXT),
            LookupState::Success(lookup_data) => text(format!(
                "Connected: {} {} version v{}",
                lookup_data.url.scheme(),
                lookup_data.url.authority(),
                lookup_data.version
            ))
            .style(Palette::DARK.success),
            LookupState::Error => text("Failed to connect").style(Palette::DARK.danger),
        };

        let target_row: Row<_> = row![target_input, target_button].spacing(SPACING);

        let remember_check = checkbox(
            "Save connection URL",
            self.remember,
            AppMessage::RememberChanged,
        )
        .text_size(16)
        .size(20)
        .spacing(SPACING);

        // Keep running notice
        let notice = text(
            "You must keep this program running while playing. \
            Closing this program will cause you to connect to the official servers instead.",
        )
        .style(RED_TEXT);

        let content: Column<_> =
            column![target_text, target_row, remember_check, status_text, notice].spacing(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(SPACING)
            .into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}
