use crate::{
    api::{try_update_host, LookupData, LookupError},
    config::ClientConfig,
    constants::{APP_VERSION, ICON_BYTES},
    patch::{try_patch_game, try_remove_patch},
};
use iced::{
    executor,
    theme::Palette,
    widget::{
        button, checkbox, column, container, row, scrollable, scrollable::Properties, text,
        text_input, Button, Column, Row, Text, TextInput,
    },
    window::{self, icon},
    Application, Color, Command, Length, Settings, Theme,
};

use super::{show_error, show_info};

/// The window size
pub const WINDOW_SIZE: (u32, u32) = (500, 310);

pub fn init(config: Option<ClientConfig>) {
    App::run(Settings {
        window: window::Settings {
            icon: icon::from_file_data(ICON_BYTES, None).ok(),
            size: WINDOW_SIZE,
            resizable: false,

            ..window::Settings::default()
        },
        flags: config,
        ..Settings::default()
    })
    .unwrap();
}

struct App {
    lookup_result: LookupState,
    remember: bool,
    target: String,
}

/// Messages used for updating the game state
#[derive(Debug, Clone)]
enum AppMessage {
    /// The redirector target address changed
    TargetChanged(String),
    /// The redirector target should be updated
    UpdateTarget,
    /// Display the patch game dialog asking the player to patch
    PatchGame,
    /// Remove the patch from the game
    RemovePatch,
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
    Error(String),
}

impl Application for App {
    type Message = AppMessage;
    type Executor = executor::Default;
    type Flags = Option<ClientConfig>;
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (target, remember) = flags
            .map(|value| (value.connection_url, true))
            .unwrap_or_default();

        (
            App {
                lookup_result: LookupState::None,
                target,
                remember,
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
                        Err(err) => LookupState::Error(err.to_string()),
                    };
                    AppMessage::LookupState(result)
                };

                // Perform the async lookup with the callback
                return Command::perform(try_update_host(target, self.remember), post_lookup);
            }
            // Patching
            AppMessage::PatchGame => match try_patch_game() {
                // Game was patched
                Ok(true) => show_info("Game patched", "Sucessfully patched game"),
                // Patching was cancelled
                Ok(false) => {}
                // Error occurred
                Err(err) => show_error("Failed to patch game", &err.to_string()),
            },
            // Patch removal
            AppMessage::RemovePatch => match try_remove_patch() {
                // Patch was removed
                Ok(true) => show_info("Patch removed", "Sucessfully removed patch"),
                // Patch removal cancelled
                Ok(false) => {}
                // Error occurred
                Err(err) => show_error("Failed to remove patch", &err.to_string()),
            },
            // Lookup result changed
            AppMessage::LookupState(value) => self.lookup_result = value,
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
                lookup_data.scheme, lookup_data.host, lookup_data.version
            ))
            .style(Palette::DARK.success),
            LookupState::Error(err) => text(err).style(Palette::DARK.danger),
        };

        let status_text = scrollable(status_text).direction(scrollable::Direction::Horizontal(
            Properties::new().width(2).scroller_width(2),
        ));

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

        // Game patching buttons
        let patch_button: Button<_> = button("Patch Game")
            .on_press(AppMessage::PatchGame)
            .padding(5);
        let unpatch_button: Button<_> = button("Remove Patch")
            .on_press(AppMessage::RemovePatch)
            .padding(5);

        // Patching notice
        let patch_notice: Text = text(
            "You must patch your game in order to make it compatible with Pocket Relay. \
            This patch can be left applied and wont affect playing on official servers.",
        )
        .style(DARK_TEXT);

        let actions_row: Row<_> = row![patch_button, unpatch_button]
            .spacing(SPACING)
            .width(Length::Fill);

        let content: Column<_> = column![
            target_text,
            target_row,
            remember_check,
            status_text,
            notice,
            patch_notice,
            actions_row
        ]
        .spacing(10);

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
