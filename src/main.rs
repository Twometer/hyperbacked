use iced::{Application, Settings};

use crate::gui::HyperbackedApp;

mod backup;
mod crypto;
mod gui;
mod passphrase;
mod printer;
mod qrcode;

fn main() -> anyhow::Result<()> {
    HyperbackedApp::run(Settings {
        window: iced::window::Settings {
            size: (700, 550),
            resizable: false,
            ..Default::default()
        },
        ..Default::default()
    })?;
    Ok(())
}
