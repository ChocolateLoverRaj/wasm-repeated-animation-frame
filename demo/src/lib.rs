use fps_counter::FPSCounter;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use wasm_react::hooks::{use_effect, use_state, Deps};
use wasm_react::{clones, export_components, h, Callback, Component, VNode};

use wasm_react::{create_context, hooks::State, Context};
use wasm_repeated_animation_frame::RafLoop;
use web_sys::console::log_1;

thread_local! {
  pub static STATE_CONTEXT: Context<Option<State<i32>>> = create_context(None.into());
}

pub struct App;

impl Component for App {
    fn render(&self) -> VNode {
        let show_sub_component = use_state(|| false);

        let value = show_sub_component.value();

        (
            h!(button)
                .on_click(&Callback::new({
                    clones!(mut show_sub_component);

                    move |_| {
                        show_sub_component.set(|show_sub_component| !show_sub_component);
                    }
                }))
                .build("Toggle sub component"),
            match *value {
                true => SubComponent {}.build(),
                false => ().into(),
            },
        )
            .into()
    }
}

impl TryFrom<JsValue> for App {
    type Error = JsValue;

    fn try_from(_: JsValue) -> Result<Self, Self::Error> {
        console_error_panic_hook::set_once();
        Ok(App)
    }
}

struct SubComponent {}
impl Component for SubComponent {
    fn render(&self) -> VNode {
        let fps = use_state(|| 0);
        use_effect(
            {
                clones!(mut fps);

                let (mut raf_loop, canceler) = RafLoop::new();
                move || {
                    spawn_local(async move {
                        let mut fps_counter = FPSCounter::new();
                        loop {
                            if !raf_loop.next().await {
                                log_1(&"break loop".into());
                                break;
                            };
                            fps.set(|_| fps_counter.tick());
                        }
                    });
                    move || {
                        spawn_local(async move {
                            canceler.cancel().await;
                            log_1(&"stopped raf loop".into());
                        });
                        log_1(&"stop raf loop".into());
                    }
                }
            },
            Deps::none(),
        );

        let fps = *fps.value();

        h!(h1).build(("FPS: ", h!(code).build(fps)))
    }
}

export_components! { App }
