use super::show_error;
use crate::{
    api::{try_update_host, LookupData, LookupError},
    config::ClientConfig,
    constants::{ICON_BYTES, WINDOW_TITLE},
};
use futures::FutureExt;
use ngd::NwgUi;
use nwg::{CheckBoxState, NativeUi};
use reqwest::Client;
use std::cell::RefCell;
use tokio::task::JoinHandle;

extern crate native_windows_derive as ngd;
extern crate native_windows_gui as nwg;

pub const WINDOW_SIZE: (i32, i32) = (500, 200);

#[derive(NwgUi, Default)]
pub struct App {
    /// Window Icon
    #[nwg_resource(source_bin: Some(ICON_BYTES))]
    icon: nwg::Icon,

    /// App window
    #[nwg_control(
        size: WINDOW_SIZE,
        position: (5, 5),
        icon: Some(&data.icon),
        title: WINDOW_TITLE,
        flags: "WINDOW|VISIBLE|MINIMIZE_BOX"
    )]
    #[nwg_events(OnWindowClose: [nwg::stop_thread_dispatch()])]
    window: nwg::Window,

    /// Grid layout for all the content
    #[nwg_layout(parent: window)]
    grid: nwg::GridLayout,

    /// Label for the connection URL input
    #[nwg_control(text: "Please put the server Connection URL below and press 'Set'")]
    #[nwg_layout_item(layout: grid, col: 0, row: 0, col_span: 2)]
    target_url_label: nwg::Label,

    /// Input for the connection URL
    #[nwg_control(focus: true)]
    #[nwg_layout_item(layout: grid, col: 0, row: 1, col_span: 2)]
    target_url_input: nwg::TextInput,

    /// Button for connecting
    #[nwg_control(text: "Set")]
    #[nwg_layout_item(layout: grid, col: 2, row: 1, col_span: 1)]
    #[nwg_events(OnButtonClick: [App::handle_set])]
    set_button: nwg::Button,

    /// Checkbox for whether to remember the connection URL
    #[nwg_control(text: "Save connection URL")]
    #[nwg_layout_item(layout: grid, col: 0, row: 2, col_span: 3)]
    remember_checkbox: nwg::CheckBox,

    /// Connection state label
    #[nwg_control(text: "Not connected")]
    #[nwg_layout_item(layout: grid, col: 0, row: 3, col_span: 3)]
    connection_label: nwg::Label,

    /// Label telling the player to keep the program running
    #[nwg_control(
        text: "You must keep this program running while playing. Closing this \n\
        program will cause you to connect to the official servers instead."
    )]
    #[nwg_layout_item(layout: grid, col: 0, row: 4, col_span: 3)]
    keep_running_label: nwg::Label,

    /// Notice for connection completion
    #[nwg_control]
    #[nwg_events(OnNotice: [App::handle_connect_notice])]
    connect_notice: nwg::Notice,

    /// Join handle for the connect task
    connect_task: RefCell<Option<JoinHandle<Result<LookupData, LookupError>>>>,

    /// Http client for sending requests
    http_client: Client,
}

impl App {
    /// Handles the "Set" button being pressed, dispatches a connect task
    /// that will wake up the App with `App::handle_connect_notice` to
    /// handle the connection result.
    fn handle_set(&self) {
        if let Some(task) = self.connect_task.take() {
            task.abort();
        }

        self.connection_label.set_text("Connecting...");
        let target = self.target_url_input.text();
        let remember = self.remember_checkbox.check_state() == CheckBoxState::Checked;
        let sender = self.connect_notice.sender();
        let http_client = self.http_client.clone();

        let task = tokio::spawn(async move {
            let result = try_update_host(http_client, target, remember).await;
            sender.notice();
            result
        });

        *self.connect_task.borrow_mut() = Some(task);
    }

    /// Handles the connection complete notice updating the UI
    /// with the new connection state from the task result
    fn handle_connect_notice(&self) {
        let result = self
            .connect_task
            .borrow_mut()
            .take()
            // Flatten on the join result
            .and_then(|task| task.now_or_never())
            // Flatten join failure errors (Out of our control)
            .and_then(|inner| inner.ok());

        let result = match result {
            Some(value) => value,
            None => {
                return;
            }
        };

        match result {
            Ok(result) => {
                let text = format!(
                    "Connected: {} {} version v{}",
                    result.scheme, result.host, result.version
                );
                self.connection_label.set_text(&text)
            }
            Err(err) => {
                self.connection_label.set_text("Failed to connect");
                show_error("Failed to connect", &err.to_string());
            }
        }
    }
}

pub fn init(config: Option<ClientConfig>, client: Client) {
    nwg::init().expect("Failed to initialize native UI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let app = App::build_ui(App {
        http_client: client,
        ..Default::default()
    })
    .expect("Failed to build native UI");

    let (target, remember) = config
        .map(|value| (value.connection_url, true))
        .unwrap_or_default();

    app.target_url_input.set_text(&target);

    if remember {
        app.remember_checkbox
            .set_check_state(CheckBoxState::Checked);
    }

    nwg::dispatch_thread_events();
}
