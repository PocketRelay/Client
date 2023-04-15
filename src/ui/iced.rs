use crate::{
    constants::{APP_VERSION, ICON_BYTES},
    remove_host_entry, show_error, show_info, try_lookup_host, try_patch_game, try_remove_patch,
    LookupData, TARGET,
};
use iced::{
    executor,
    theme::Palette,
    widget::{
        button, column, container, row, text, text_input, Button, Column, Row, Text, TextInput,
    },
    window::{self, Icon},
    Application, Color, Command, Length, Settings, Theme,
};

/// The window size
pub const WINDOW_SIZE: (u32, u32) = (500, 280);

pub fn init(runtime: tokio::runtime::Runtime) {
    let _guard = runtime.enter();

    App::run(Settings {
        window: window::Settings {
            icon: Icon::from_file_data(ICON_BYTES, None).ok(),
            size: WINDOW_SIZE,
            resizable: false,

            ..window::Settings::default()
        },

        ..Settings::default()
    })
    .unwrap();
}

struct App {
    lookup_result: LookupResult,
    target: String,
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = remove_host_entry();
    }
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
    LookupResult(LookupResult),
}

/// Different outcomes and states for looking up
#[derive(Debug, Clone)]
enum LookupResult {
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
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            App {
                lookup_result: LookupResult::None,
                target: "".to_string(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("Pocket Relay Client v{}", APP_VERSION)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMessage::TargetChanged(value) => {
                self.target = value;
            }
            AppMessage::UpdateTarget => {
                if let LookupResult::Loading = self.lookup_result {
                    return Command::none();
                }

                self.lookup_result = LookupResult::Loading;

                let target = self.target.clone();
                return Command::perform(tokio::spawn(try_lookup_host(target)), |result| {
                    let result = match result {
                        Ok(Ok(value)) => {
                            let write = &mut *TARGET.blocking_write();
                            *write = Some(value.clone());
                            LookupResult::Success(value)
                        }
                        Ok(Err(err)) => LookupResult::Error(err.to_string()),
                        Err(_) => LookupResult::Error("Failed to handle request".to_string()),
                    };
                    AppMessage::LookupResult(result)
                });
            }
            AppMessage::PatchGame => match try_patch_game() {
                Ok(true) => show_info("Game patched", "Sucessfully patched game"),
                Ok(false) => {}
                Err(err) => show_error("Failed to patch game", &err.to_string()),
            },
            AppMessage::RemovePatch => match try_remove_patch() {
                Ok(true) => show_info("Patch removed", "Sucessfully removed patch"),
                Ok(false) => {}
                Err(err) => show_error("Failed to remove patch", &err.to_string()),
            },
            AppMessage::LookupResult(value) => {
                self.lookup_result = value;
            }
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        const DARK_TEXT: Color = Color::from_rgb(0.4, 0.4, 0.4);
        const RED_TEXT: Color = Color::from_rgb(0.8, 0.4, 0.4);
        const YELLOW_TEXT: Color = Color::from_rgb(0.8, 0.8, 0.4);
        const ORANGE_TEXT: Color = Color::from_rgb(0.8, 0.6, 0.4);
        const SPACING: u16 = 10;

        let target_input: TextInput<_> =
            text_input("Connection URL", &self.target, AppMessage::TargetChanged).padding(10);

        let target_text: Text =
            text("Please put the server Connection URL below and press 'Set'").style(DARK_TEXT);
        let target_button: Button<_> = button("Set").on_press(AppMessage::UpdateTarget).padding(10);

        let status_text: Text = match &self.lookup_result {
            LookupResult::None => text("Not Connected.").style(ORANGE_TEXT),
            LookupResult::Loading => text("Connecting...").style(YELLOW_TEXT),
            LookupResult::Success(lookup_data) => text(format!(
                "Connected: {} {} version v{}",
                lookup_data.scheme, lookup_data.host, lookup_data.version
            ))
            .style(Palette::DARK.success),
            LookupResult::Error(err) => text(err).style(Palette::DARK.danger),
        };

        let target_row: Row<_> = row![target_input, target_button].spacing(SPACING);

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
