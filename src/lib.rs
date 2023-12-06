use async_channel::{Receiver, Sender};
use futures::{future::select_all, FutureExt};
use js_sys::Function;
use manual_future::ManualFuture;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

pub struct RafLoopCanceler {
    sender: Sender<()>,
}

impl RafLoopCanceler {
    pub async fn cancel(&self) {
        self.sender.send(()).await.unwrap();
    }
}

pub struct RafLoop {
    handle: Option<i32>,
    receiver: Receiver<()>,
}

impl RafLoop {
    pub fn new() -> (RafLoop, RafLoopCanceler) {
        let (sender, receiver) = async_channel::bounded::<()>(1);
        (
            RafLoop {
                handle: None,
                receiver,
            },
            RafLoopCanceler { sender },
        )
    }

    pub fn cancel(&mut self) {
        window()
            .unwrap()
            .cancel_animation_frame(self.handle.unwrap())
            .unwrap();
        self.handle = None;
    }

    pub async fn next(&mut self) -> bool {
        let (future, completer) = ManualFuture::<()>::new();
        let closure = Closure::once(Box::new(move || {
            spawn_local(completer.complete(()));
        }) as Box<dyn FnOnce()>);
        let handle = window()
            .unwrap()
            .request_animation_frame(&(closure.into_js_value().unchecked_into::<Function>()))
            .unwrap();
        self.handle = Some(handle);
        enum Event {
            Stop,
            Raf,
        }
        let (event, _, _) = select_all(vec![
            async {
                future.await;
                Event::Raf
            }
            .boxed(),
            async {
                self.receiver.recv().await.unwrap();
                Event::Stop
            }
            .boxed(),
        ])
        .await;
        match event {
            Event::Raf => true,
            Event::Stop => false,
        }
    }
}
