//! Login registration is an explicit preference backed by macOS Service Management.
use super::QuickNoteApp;
use crate::i18n;
use eframe::egui::Checkbox;
use eframe::egui::Ui;
use objc2_foundation::NSBundle;
use smappservice_rs::AppService;
use smappservice_rs::ServiceStatus;
use smappservice_rs::ServiceType;

#[derive(Default)]
pub(super) struct LoginItem {
    status: Option<ServiceStatus>,
}

impl LoginItem {
    fn refresh(&mut self) {
        self.status = NSBundle::mainBundle()
            .bundleIdentifier()
            .filter(|id| id.to_string() == "local.quicknote.mac")
            .map(|_| AppService::new(ServiceType::MainApp).status());
    }
}

impl QuickNoteApp {
    pub(super) fn refresh_login_item(&mut self) {
        self.capture.login.refresh();
    }

    pub(super) fn login_option(&mut self, ui: &mut Ui) {
        ui.add_space(12.0);
        let pending = matches!(
            self.capture.login.status,
            Some(ServiceStatus::RequiresApproval)
        );
        let mut enabled =
            pending || matches!(self.capture.login.status, Some(ServiceStatus::Enabled));
        let response = ui.add_enabled(
            // NotFound does not establish whether the first registration can succeed.
            self.capture.login.status.is_some(),
            Checkbox::new(&mut enabled, &self.locale.text().capture.open_at_login)
                .indeterminate(pending),
        );
        crate::ui::focus_ring(ui, &response, self.theme.palette());
        if response.changed() {
            let service = AppService::new(ServiceType::MainApp);
            let result = if enabled {
                service.register()
            } else {
                service.unregister()
            };
            self.refresh_login_item();
            if let Err(error) = result {
                self.error = Some(
                    i18n::message(|m| &m.capture.error_login)
                        .cause("error", error)
                        .into(),
                );
            }
        }
        match self.capture.login.status {
            None => {
                ui.weak(&self.locale.text().capture.login_unavailable);
            }
            Some(ServiceStatus::RequiresApproval) => {
                ui.weak(&self.locale.text().capture.login_requires_approval);
                if ui
                    .button(&self.locale.text().capture.login_settings)
                    .clicked()
                {
                    AppService::open_system_settings_login_items();
                }
            }
            _ => {}
        }
    }

    #[cfg(feature = "diagnostics")]
    #[must_use]
    pub fn login_status(&self) -> &'static str {
        match self.capture.login.status {
            None => "unavailable",
            Some(ServiceStatus::NotRegistered) => "not_registered",
            Some(ServiceStatus::Enabled) => "enabled",
            Some(ServiceStatus::RequiresApproval) => "requires_approval",
            Some(ServiceStatus::NotFound) => "not_found",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::NodeT;
    use egui_kittest::kittest::Queryable;

    #[test]
    fn a_bundled_app_can_request_its_first_registration_when_status_is_not_found()
    -> crate::storage::Result<()> {
        for status in [
            Some(ServiceStatus::NotFound),
            Some(ServiceStatus::NotRegistered),
            Some(ServiceStatus::RequiresApproval),
            Some(ServiceStatus::Enabled),
            None,
        ] {
            let directory = tempfile::tempdir()?;
            let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
            app.capture.login.status = status;
            let mut harness = egui_kittest::Harness::builder().build_ui(|ui| app.login_option(ui));
            harness.run();
            assert_eq!(
                harness
                    .get_by_label("Open at login")
                    .accesskit_node()
                    .is_disabled(),
                status.is_none()
            );
            // Rendering never registers or unregisters a real login item.
        }
        Ok(())
    }
}
