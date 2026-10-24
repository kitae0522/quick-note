use eframe::egui::CentralPanel;
use eframe::egui::Frame;
use eframe::egui::Theme as GuiTheme;
use eframe::egui::Ui;
use eframe::egui::Visuals;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quick_note::appearance;
use quick_note::appearance::Theme;
use quick_note::model::NoteSummary;
use quick_note::ui::header;
use quick_note::ui::header::HeaderState;

struct HeaderApp {
    state: HeaderState,
    tabs: Vec<NoteSummary>,
    theme: Theme,
    windows: bool,
}

impl eframe::App for HeaderApp {
    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        CentralPanel::default().frame(Frame::NONE).show(ui, |ui| {
            let _output = header::header(
                ui,
                &mut self.state,
                &self.tabs,
                "0",
                self.theme.palette(),
                if self.windows {
                    quick_note::ui::header::WindowControls::Windows
                } else {
                    quick_note::ui::header::WindowControls::Mac
                },
                quick_note::i18n::english(),
            );
        });
    }
    fn clear_color(&self, _: &Visuals) -> [f32; 4] {
        self.theme.palette().toolbar.to_normalized_gamma_f32()
    }
}

#[test]
fn narrow_header_keeps_the_active_tab_delete_control_clear_of_other_controls() {
    for (windows, height) in [false, true]
        .into_iter()
        .flat_map(|windows| [32.0, 40.0, 56.0].map(|height| (windows, height)))
    {
        let app = HeaderApp {
            state: HeaderState {
                reveal_active: true,
                ..Default::default()
            },
            tabs: [
                "A thought worth keeping",
                "Release checklist",
                "Weekend ideas",
            ]
            .into_iter()
            .enumerate()
            .map(|(index, title)| NoteSummary {
                is_blank: false,
                id: index.to_string(),
                title: title.into(),
                updated_at: 0.0,
            })
            .collect(),
            theme: Theme::builtin("helium"),
            windows,
        };
        let mut harness = Harness::builder()
            .with_size([320.0, height])
            .with_theme(GuiTheme::Light)
            .with_render_cursor(false)
            .build_eframe(move |creation| {
                appearance::install_fonts(&creation.egui_ctx);
                app.theme.apply(&creation.egui_ctx);
                app
            });
        harness.run();
        let close = harness
            .get_by_label("Delete A thought worth keeping")
            .rect();
        let new = harness.get_by_label("New note").rect();
        assert!(
            close.left() >= 0.0 && close.right() <= new.left(),
            "active delete control must be visible and not overlap New note: {close:?} / {new:?}"
        );
        for label in ["Find a note", "New note", "Open settings"] {
            let rect = harness.get_by_label(label).rect();
            assert!(
                rect.left() >= if windows { 16.0 } else { 92.0 } && rect.right() <= 304.0,
                "controls must stay inside the header safe area: {label}: {rect:?}"
            );
            assert!(rect.width() >= 24.0 && rect.height() >= 24.0);
            assert!(rect.top() >= 0.0 && rect.bottom() <= height);
        }
        if windows {
            for label in ["Minimize window", "Maximize window", "Close window"] {
                let rect = harness.get_by_label(label).rect();
                assert!(rect.left() >= 16.0 && rect.right() <= 304.0);
            }
        }
        std::fs::create_dir_all(".artifacts/rust-ui")
            .expect("isolated verification fixture should be valid");
        harness
            .render()
            .expect("isolated verification fixture should be valid")
            .save(format!(
                ".artifacts/rust-ui/header-320-{}-{height:.0}.png",
                if windows { "windows" } else { "macos" }
            ))
            .expect("isolated verification fixture should be valid");
    }
}
