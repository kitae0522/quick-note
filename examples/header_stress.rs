//! Fixture-only visual report of the production header. No production data or network access.
use eframe::egui::CentralPanel;
use eframe::egui::Frame;
use eframe::egui::Theme as GuiTheme;
use eframe::egui::Ui;
use eframe::egui::Visuals;
use egui_kittest::Harness;
use quick_note::appearance;
use quick_note::appearance::Theme;
use quick_note::model::NoteSummary;
use quick_note::ui::header;
use quick_note::ui::header::HeaderState;
use std::fs;
use std::path::Path;
use std::sync::Arc;

struct HeaderFixture {
    theme: Theme,
    tabs: Vec<NoteSummary>,
    active: String,
    state: HeaderState,
    windows: bool,
}

impl eframe::App for HeaderFixture {
    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        CentralPanel::default().frame(Frame::NONE).show(ui, |ui| {
            let _output = header::header(
                ui,
                &mut self.state,
                &self.tabs,
                &self.active,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = Path::new(".artifacts/header-stress");
    fs::create_dir_all(output)?;
    let typical = vec![
        "A thought worth keeping".to_owned(),
        "Release checklist".into(),
        "Weekend ideas".into(),
    ];
    let many: Vec<_> = (1..=30).map(|index| format!("Idea {index:03}")).collect();
    let long = "An idea that starts small. Then it becomes a very long title with \
            several sentences and enough words to fill the whole window.";
    let scenarios = vec![
        ("One blank draft", vec!["Untitled".into()], 920.0, 0, false),
        ("One word", vec!["Idea".into()], 920.0, 0, false),
        ("Three typical notes", typical.clone(), 920.0, 0, false),
        (
            "Several sentences",
            vec![long.into(), "Next idea".into()],
            920.0,
            0,
            false,
        ),
        (
            "One unbreakable title",
            vec![
                format!("https://ideas.example/{}", "a".repeat(180)),
                "Next idea".into(),
            ],
            920.0,
            0,
            false,
        ),
        (
            "Emoji and Korean",
            vec![
                "💡".into(),
                "아이디어 💡 — 작은 시작".into(),
                "Coffee ☕ and a walk".into(),
            ],
            920.0,
            1,
            false,
        ),
        (
            "RTL and mixed direction",
            vec![
                "فكرة جديدة".into(),
                "רעיון חדש".into(),
                "Quick Note — فكرة جديدة".into(),
            ],
            920.0,
            2,
            false,
        ),
        (
            "Accents and tall scripts",
            vec![
                "Café e\u{301} Ångström".into(),
                "Tiếng Việt — ý tưởng".into(),
                "ความคิดใหม่".into(),
            ],
            920.0,
            1,
            false,
        ),
        ("Thirty tabs · first active", many.clone(), 920.0, 0, false),
        ("Thirty tabs · last active", many, 920.0, 29, false),
        (
            "320 px · long title",
            vec![long.into(), "Next idea".into(), "Weekend".into()],
            320.0,
            0,
            false,
        ),
        ("1440 px · wide window", typical.clone(), 1440.0, 0, false),
        ("Windows controls · 320 px", typical.clone(), 320.0, 0, true),
        ("Windows controls · 920 px", typical, 920.0, 0, true),
    ];
    let mut sections = String::new();
    for (index, (label, titles, width, selected, windows)) in scenarios.into_iter().enumerate() {
        let tabs: Vec<_> = titles
            .into_iter()
            .enumerate()
            .map(|(index, title)| NoteSummary {
                is_blank: false,
                id: index.to_string(),
                title,
                updated_at: 0.0,
            })
            .collect();
        let active = tabs[selected].id.clone();
        let fixture = HeaderFixture {
            theme: Theme::builtin("helium"),
            tabs,
            active,
            windows,
            state: HeaderState {
                reveal_active: true,
                ..Default::default()
            },
        };
        let mut harness = Harness::builder()
            .with_size([width, 40.0])
            .with_theme(GuiTheme::Light)
            .with_render_cursor(false)
            .with_max_steps(12)
            .build_eframe(move |creation| {
                appearance::install_fonts(&creation.egui_ctx);
                fixture.theme.apply(&creation.egui_ctx);
                creation.egui_ctx.add_font_provider(Arc::new(
                    egui_system_font_provider::SystemFontProvider::new(),
                ));
                fixture
            });
        harness.run();
        let file = output.join(format!("{index:02}.png"));
        harness.render()?.save(&file)?;
        sections.push_str(&format!(
            "<section><p>{label} <span>· {width:.0} \
                px</span></p><div class=sample><img src=\"{}\" width=\"{width}\" \
                height=40 alt=\"Native header: {label}\"></div><small \
                data-scenario=\"{index}\">Pending visual observation</small></section>",
            fs::canonicalize(file)?.display()
        ));
    }
    let html = format!(include_str!("header_report.html"), sections = sections);
    fs::write(output.join("index.html"), html)?;
    println!("{}", fs::canonicalize(output.join("index.html"))?.display());
    Ok(())
}
