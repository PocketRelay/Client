use crate::{
    api::try_update_host,
    config::ClientConfig,
    constants::{APP_VERSION, ICON_BYTES},
    patch::{try_patch_game, try_remove_patch},
};
use ngw::{CheckBoxState, GridLayoutItem, Icon};

extern crate native_windows_gui as ngw;

pub const WINDOW_SIZE: (i32, i32) = (500, 300);

pub fn init(runtime: tokio::runtime::Runtime, config: Option<ClientConfig>) {
    ngw::init().expect("Failed to initialize native UI");
    ngw::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let (target, remember) = config
        .map(|value| (value.connection_url, true))
        .unwrap_or_default();

    let mut window = Default::default();
    let mut target_url = Default::default();
    let mut set_button = Default::default();
    let mut remember_checkbox = Default::default();
    let mut p_button = Default::default();
    let mut pr_button = Default::default();
    let layout = Default::default();

    let mut top_label = Default::default();
    let mut mid_label = Default::default();
    let mut bot_label = Default::default();
    let mut c_label = Default::default();

    let mut icon = Default::default();

    Icon::builder()
        .source_bin(Some(ICON_BYTES))
        .build(&mut icon)
        .unwrap();

    // Create window
    ngw::Window::builder()
        .size(WINDOW_SIZE)
        .position((5, 5))
        .icon(Some(&icon))
        .title(&format!("Pocket Relay Client v{}", APP_VERSION))
        .build(&mut window)
        .unwrap();

    // Create information text
    ngw::Label::builder()
        .text("Please put the server Connection URL below and press 'Set'")
        .parent(&window)
        .build(&mut top_label)
        .unwrap();
    ngw::Label::builder()
        .text("You must keep this program running while playing. Closing this \nprogram will cause you to connect to the official servers instead.")
        .parent(&window)
        .build(&mut mid_label)
        .unwrap();
    ngw::Label::builder()
        .text("You must patch your game in order to make it compatible with\n Pocket Relay")
        .parent(&window)
        .build(&mut bot_label)
        .unwrap();
    ngw::Label::builder()
        .text("Not connected")
        .parent(&window)
        .build(&mut c_label)
        .unwrap();

    // Create the url input and set button
    ngw::TextInput::builder()
        .text(&target)
        .focus(true)
        .parent(&window)
        .build(&mut target_url)
        .unwrap();
    ngw::Button::builder()
        .text("Set")
        .parent(&window)
        .build(&mut set_button)
        .unwrap();
    ngw::CheckBox::builder()
        .text("Save connection URL")
        .check_state(if remember {
            CheckBoxState::Checked
        } else {
            CheckBoxState::Unchecked
        })
        .parent(&window)
        .build(&mut remember_checkbox)
        .unwrap();

    // Create the patch buttons
    ngw::Button::builder()
        .text("Patch Game")
        .parent(&window)
        .build(&mut p_button)
        .unwrap();
    ngw::Button::builder()
        .text("Remove Patch")
        .parent(&window)
        .build(&mut pr_button)
        .unwrap();

    // Create the layout grid for the UI
    ngw::GridLayout::builder()
        .parent(&window)
        .child_item(GridLayoutItem::new(&top_label, 0, 0, 2, 1))
        .child_item(GridLayoutItem::new(&target_url, 0, 1, 2, 1))
        .child_item(GridLayoutItem::new(&set_button, 0, 2, 2, 1))
        .child_item(GridLayoutItem::new(&remember_checkbox, 0, 3, 2, 1))
        .child_item(GridLayoutItem::new(&c_label, 0, 4, 2, 1))
        .child_item(GridLayoutItem::new(&mid_label, 0, 5, 2, 1))
        .child_item(GridLayoutItem::new(&p_button, 0, 6, 1, 1))
        .child_item(GridLayoutItem::new(&pr_button, 1, 6, 1, 1))
        .child_item(GridLayoutItem::new(&bot_label, 0, 7, 2, 1))
        .build(&layout)
        .unwrap();

    let window_handle = window.handle;

    let handler = ngw::full_bind_event_handler(&window_handle, move |event, _evt_data, handle| {
        use ngw::Event as E;

        match event {
            E::OnWindowClose => {
                if &handle == &window_handle {
                    ngw::stop_thread_dispatch();
                }
            }

            E::OnButtonClick => {
                if &handle == &set_button {
                    c_label.set_text("Loading...");

                    let target = target_url.text();

                    let result = runtime.block_on(try_update_host(
                        target,
                        remember_checkbox.check_state() == CheckBoxState::Checked,
                    ));

                    match result {
                        Ok(value) => c_label.set_text(&format!(
                            "Connected: {} {} version v{}",
                            value.scheme, value.host, value.version
                        )),
                        Err(err) => {
                            c_label.set_text("Failed to connect");
                            ngw::modal_error_message(
                                &window_handle,
                                "Failed to connect",
                                &err.to_string(),
                            );
                        }
                    }
                } else if &handle == &p_button {
                    match try_patch_game() {
                        Ok(true) => {
                            ngw::modal_info_message(
                                &window_handle,
                                "Game patched",
                                "Sucessfully patched game",
                            );
                        }
                        Ok(false) => {}
                        Err(err) => {
                            ngw::modal_error_message(
                                &window_handle,
                                "Failed to patch game",
                                &err.to_string(),
                            );
                        }
                    }
                } else if &handle == &pr_button {
                    match try_remove_patch() {
                        Ok(true) => {
                            ngw::modal_info_message(
                                &window_handle,
                                "Patch removed",
                                "Sucessfully removed patch",
                            );
                        }
                        Ok(false) => {}
                        Err(err) => {
                            ngw::modal_error_message(
                                &window_handle,
                                "Failed to remove patch",
                                &err.to_string(),
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    });

    ngw::dispatch_thread_events();
    ngw::unbind_event_handler(&handler);
}
