//! Observable progress, interruption and repaint behavior of native UI transitions.
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::RawInput;
use quick_note::ui::motion;
use std::time::Duration;

fn sample(context: &Context, frame: u32, open: bool, reveal: bool) -> (f32, Duration) {
    let mut value = 0.0;
    let input = RawInput {
        time: Some(f64::from(frame) / 60.0),
        ..Default::default()
    };
    let mut output = context.run_ui(input, |_| {
        value = if reveal {
            motion::reveal(context, Id::unique("test-motion"), open)
        } else {
            motion::transition(context, Id::unique("test-motion"), open)
        };
    });
    let repaint = output
        .viewport_output
        .values()
        .next()
        .expect("root viewport")
        .repaint_delay;
    output.textures_delta.clear();
    (value, repaint)
}

#[test]
fn first_render_and_reduced_motion_never_wait_for_a_transition() {
    let context = Context::default();
    motion::configure(&context, false);
    assert_eq!(sample(&context, 0, true, false).0, 1.0);
    motion::configure(&context, true);
    assert_eq!(sample(&context, 1, false, false).0, 0.0);
    assert_eq!(sample(&context, 2, true, false).0, 1.0);
    assert_eq!(sample(&context, 3, false, false).0, 0.0);
}

#[test]
fn an_interrupted_transition_reverses_without_jumping_and_settles_within_150_ms() {
    let context = Context::default();
    motion::configure(&context, false);
    assert_eq!(sample(&context, 0, false, false).0, 0.0);
    let first = sample(&context, 1, true, false).0;
    let second = sample(&context, 2, true, false).0;
    assert!(0.0 < first && first < second && second < 1.0);
    let reversed = sample(&context, 3, false, false).0;
    assert!(reversed < second && reversed > 0.0);
    assert!(
        (second - reversed).abs() < 0.3,
        "reversal must not switch easing coordinate spaces"
    );
    for frame in 4..=12 {
        sample(&context, frame, true, false);
    }
    assert_eq!(sample(&context, 13, true, false).0, 1.0);
    assert_ne!(
        sample(&context, 14, true, false).1,
        Duration::ZERO,
        "settled motion must not keep the app repainting"
    );
}

#[test]
fn closing_a_surface_cancels_its_motion_and_an_immediate_reopen_is_safe() {
    let context = Context::default();
    motion::configure(&context, false);
    sample(&context, 0, false, true);
    assert!(sample(&context, 1, true, true).0 > 0.0);
    assert_eq!(sample(&context, 2, false, true).0, 0.0);
    assert!(sample(&context, 3, true, true).0 > 0.0);
    motion::configure(&context, true);
    assert_eq!(sample(&context, 4, true, true).0, 1.0);
}
