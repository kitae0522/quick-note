use super::Theme;
use crate::storage::Result;
use eframe::egui::Color32;

#[test]
fn preferred_font_presets_lead_the_list_without_hiding_installed_fonts() {
    let mut choices = super::FontChoices::default();
    let installed = ["Arial".into(), "Custom Mono".into(), "Menlo".into()];
    choices.update(Some(&installed), "");
    assert_eq!(
        &choices.names()[..5],
        ["SF Pro", "Pretendard", "Noto Sans CJK", "Menlo", "Consolas"]
    );
    assert_eq!(choices.names().len(), 7);
    choices.update(Some(&installed), "Custom");
    assert_eq!(choices.names(), ["Custom Mono"]);
}
fn contrast(a: Color32, b: Color32) -> f32 {
    let luminance = |color: Color32| {
        let channels = color.to_array().map(|channel| {
            let value = channel as f32 / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        });
        0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]
    };
    let a = luminance(a);
    let b = luminance(b);
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
#[test]
fn builtins_have_measured_text_and_focus_contrast() {
    for id in ["helium", "graphite"] {
        let theme = Theme::builtin(id);
        let palette = theme.palette();
        for background in [
            palette.background,
            palette.toolbar,
            palette.selection,
            palette.current_line,
        ] {
            println!(
                "{id} foreground: {:.3}:1",
                contrast(palette.foreground, background)
            );
            assert!(
                contrast(palette.foreground, background) >= 4.5,
                "{id}: foreground contrast"
            );
        }
        for background in [palette.background, palette.toolbar, palette.selection] {
            let measured = contrast(palette.secondary, background);
            println!("{id} secondary: {measured:.3}:1");
            assert!(
                measured >= 4.5,
                "{id}: secondary text contrast is {measured}"
            );
            assert!(
                contrast(palette.accent, background) >= 3.0,
                "{id}: focus contrast"
            );
        }
        assert!(contrast(palette.danger, palette.on_danger) >= 4.5);
    }
}
#[test]
fn shared_theme_is_utf8_and_round_trips() -> Result<()> {
    let theme = Theme::builtin("helium");
    let mut shared = theme.clone();
    shared.name = "아이디어 💡".into();
    let bytes = serde_json::to_vec_pretty(&shared)?;
    assert!(std::str::from_utf8(&bytes).is_ok());
    assert_eq!(Theme::decode(&bytes)?, shared);
    assert!(Theme::decode(&[0xff, 0xfe]).is_err());
    assert!(Theme::decode(&vec![b' '; 65537]).is_err());
    Ok(())
}

#[test]
fn shared_themes_accept_local_font_names_without_a_bundled_whitelist() -> Result<()> {
    let mut theme = Theme::builtin("helium");
    theme.typography.font_family = "Menlo".into();
    let bytes = serde_json::to_vec(&theme)?;
    assert_eq!(Theme::decode(&bytes)?, theme);
    Ok(())
}

#[test]
fn current_line_color_is_optional_for_existing_themes_and_validated_when_present() -> Result<()> {
    for id in ["helium", "graphite"] {
        let mut source = serde_json::to_value(Theme::builtin(id))?;
        source["colors"]
            .as_object_mut()
            .expect("theme color object")
            .remove("currentLine");
        let legacy = Theme::decode(&serde_json::to_vec(&source)?)?;
        assert!(legacy.colors.current_line.is_none());
        assert!(contrast(legacy.palette().foreground, legacy.palette().current_line) >= 4.5);
        assert_eq!(Theme::decode(&serde_json::to_vec(&legacy)?)?, legacy);
        source["colors"]["currentLine"] = "#102030".into();
        let custom = Theme::decode(&serde_json::to_vec(&source)?)?;
        assert_eq!(custom.palette().current_line, Color32::from_rgb(16, 32, 48));
        source["colors"]["currentLine"] = "not a color".into();
        assert!(Theme::decode(&serde_json::to_vec(&source)?).is_err());
    }
    Ok(())
}
