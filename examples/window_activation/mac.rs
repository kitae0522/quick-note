//! Test requested activation without a renderer, notes, input injection or focus retries.
use objc2::MainThreadMarker;
use objc2_app_kit::NSApplication;
use quick_note::storage::Result;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::StartCause;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::window::WindowId;

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Hidden,
    InactiveBeforeFirstShow,
    FirstShow,
}

struct ActivationCheck {
    active: bool,
    allow_activation: bool,
    output: PathBuf,
    window: Option<Window>,
    deadline: Option<Instant>,
    active_before_show: bool,
    passed: Option<bool>,
    phase: Phase,
}

fn app_active() -> bool {
    let main = MainThreadMarker::new().expect("AppKit event loop executes on the main thread");
    NSApplication::sharedApplication(main).isActive()
}

impl ApplicationHandler for ActivationCheck {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("Quick Note activation check")
                    .with_inner_size(LogicalSize::new(320.0, 240.0))
                    .with_visible(false)
                    .with_active(self.active),
            )
            .expect("native fixture window is available");
        self.window = Some(window);
        self.deadline = Some(Instant::now() + Duration::from_millis(500));
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.deadline.expect("set above")));
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: StartCause) {
        if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            let active = app_active();
            let window = self
                .window
                .as_ref()
                .expect("window created before deadline");
            if self.phase == Phase::Hidden {
                // Settle launch activation before simulating a renderer's hidden interval.
                let main = MainThreadMarker::new().expect("AppKit callback is on the main thread");
                #[allow(deprecated)]
                NSApplication::sharedApplication(main).deactivate();
                self.phase = Phase::InactiveBeforeFirstShow;
                self.deadline = Some(Instant::now() + Duration::from_millis(500));
                event_loop
                    .set_control_flow(ControlFlow::WaitUntil(self.deadline.expect("set above")));
                return;
            }
            if self.phase == Phase::InactiveBeforeFirstShow {
                self.active_before_show = app_active();
                window.set_visible(true);
                self.phase = Phase::FirstShow;
                self.deadline = Some(Instant::now() + Duration::from_millis(500));
                event_loop
                    .set_control_flow(ControlFlow::WaitUntil(self.deadline.expect("set above")));
                return;
            }
            let expected = self.active && self.allow_activation;
            let passed =
                !self.active_before_show && active == expected && (!expected || window.has_focus());
            self.passed = Some(passed);
            let report = json!({"requested_active":self.active,"app_active_before_show":self.active_before_show,
                "allow_activation":self.allow_activation,
                "app_active_after_show":active,"window_focused":window.has_focus(),
                "window_visible":window.is_visible(),"window_minimized":window.is_minimized(),
                "expected_app_active":expected,
                "passed":passed,
                "input":"Own app deactivated before first reveal; no focus retries or scripted text"});
            fs::write(
                &self.output,
                serde_json::to_vec_pretty(&report).expect("serializable report"),
            )
            .expect("isolated evidence folder is writable");
            self.deadline = None;
            event_loop.exit();
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, _event: WindowEvent) {}
}

pub(super) fn run() -> Result<()> {
    use winit::platform::macos::EventLoopBuilderExtMacOS as _;
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let mode = arguments
        .first()
        .expect("active, inactive or no-activate is required");
    let active = mode != "inactive";
    let allow_activation = mode == "active";
    let output = PathBuf::from(arguments.get(1).expect("output JSON path is required"));
    let mut builder = EventLoop::builder();
    builder.with_activate_ignoring_other_apps(allow_activation);
    let event_loop = builder.build()?;
    let mut check = ActivationCheck {
        active,
        allow_activation,
        output,
        window: None,
        deadline: None,
        active_before_show: false,
        passed: None,
        phase: Phase::Hidden,
    };
    event_loop.run_app(&mut check)?;
    if check.passed == Some(true) {
        Ok(())
    } else {
        Err("Native window activation did not match the requested state".into())
    }
}
