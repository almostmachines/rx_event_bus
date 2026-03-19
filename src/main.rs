use rxrust::prelude::*;
use std::{cell::RefCell, convert::Infallible, rc::Rc};

#[derive(Clone, Debug)]
enum AppEvent {
    SaveRequested,
    SaveFinished { ok: bool },
}

type AppBusInner = LocalSubject<'static, AppEvent, Infallible>;

#[derive(Clone)]
struct EventBus {
    inner: AppBusInner,
}

impl EventBus {
    fn new() -> Self {
        Self {
            inner: Local::subject::<AppEvent, Infallible>(),
        }
    }

    fn publish(&self, event: AppEvent) {
        let mut subject = self.inner.clone();
        subject.next(event);
    }

    fn subscribe<F>(&self, handler: F) -> impl Subscription + use<F>
    where
        F: FnMut(AppEvent) + 'static,
    {
        self.inner.clone().subscribe(handler)
    }
}

struct Editor {
    bus: EventBus,
}

impl Editor {
    fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    fn request_save(&self) {
        self.bus.publish(AppEvent::SaveRequested);
    }

    fn finish_save(&self, ok: bool) {
        self.bus.publish(AppEvent::SaveFinished { ok });
    }
}

struct StatusPanel {
    status_text: Rc<RefCell<String>>,
    _subscription: SubscriptionGuard<BoxedSubscription>,
}

impl StatusPanel {
    fn new(bus: EventBus) -> Self {
        let status_text = Rc::new(RefCell::new(String::from("Idle")));
        let status_text_for_handler = status_text.clone();

        let subscription = bus
            .subscribe(move |event| match event {
                AppEvent::SaveRequested => {
                    *status_text_for_handler.borrow_mut() = "Saving...".to_string();
                }
                AppEvent::SaveFinished { ok } => {
                    *status_text_for_handler.borrow_mut() =
                        if ok { "Save succeeded" } else { "Save failed" }.to_string();
                }
            })
            .into_boxed()
            .unsubscribe_when_dropped();

        Self {
            status_text,
            _subscription: subscription,
        }
    }

    fn render(&self) {
        println!("status: {}", self.status_text.borrow());
    }
}

fn main() {
    let bus = EventBus::new();

    let editor = Editor::new(bus.clone());
    let status_panel = StatusPanel::new(bus.clone());

    editor.request_save();
    status_panel.render();

    editor.finish_save(true);
    status_panel.render();
}
