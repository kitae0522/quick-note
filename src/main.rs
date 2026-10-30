#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use quick_note::app;
use quick_note::app::QuickNoteApp;
use quick_note::i18n;

fn main() -> eframe::Result {
    let mut arguments = std::env::args_os().skip(1);
    let mut directory = None;
    while let Some(argument) = arguments.next() {
        if argument == "--data-directory" {
            let Some(path) = arguments.next() else {
                startup_error(&i18n::message(|m| &m.error_data_argument));
                return Ok(());
            };
            directory = Some(std::path::PathBuf::from(path));
        }
    }
    let directory = match directory.map(Ok).unwrap_or_else(app::data_directory) {
        Ok(directory) => directory,
        Err(error) => {
            startup_error(error.as_ref());
            return Ok(());
        }
    };
    let mut app = match QuickNoteApp::try_new(directory) {
        Ok(app) => app,
        Err(error) => {
            startup_error(error.as_ref());
            return Ok(());
        }
    };
    let title = app.locale.text().app_name.clone();
    eframe::run_native(
        &title,
        quick_note::native::options(&title),
        Box::new(move |context| {
            app.configure(&context.egui_ctx);
            app.install_menu(&context.egui_ctx);
            app.install_capture(&context.egui_ctx);
            app.install_resident(&context.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}

fn startup_error(error: &(dyn std::error::Error + 'static)) {
    let locale = i18n::Localizer::default();
    rfd::MessageDialog::new()
        .set_title(&locale.text().startup_title)
        .set_description(i18n::format(
            &locale.text().startup_body,
            &[("error", &locale.error(error))],
        ))
        .set_level(rfd::MessageLevel::Error)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}
