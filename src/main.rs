use std::{cell::RefCell, rc::Rc};
use rxrust::prelude::*;

#[derive(Clone, Debug)]
enum AppEvent {
    SaveRequested,
    SaveFinished { ok: bool },
}

struct SaveStats {
    successful_saves: Rc<RefCell<u32>>,
    _subscription: SubscriptionGuard<BoxedSubscription>,
}

impl SaveStats {
    fn new(bus: rxrust_event_bus::EventBus<AppEvent>) -> Self {
        let successful_saves = Rc::new(RefCell::new(0));
        let successful_saves_for_handler = successful_saves.clone();

        let subscription = bus
            .events()
            .filter_map(|event| match event {
                AppEvent::SaveFinished { ok: true } => Some(1u32),
                _ => None,
            })
            .scan(0u32, |count, inc| count + inc)
            .subscribe(move |count| {
                *successful_saves_for_handler.borrow_mut() = count;
            })
            .into_boxed()
            .unsubscribe_when_dropped();

        Self {
            successful_saves,
            _subscription: subscription,
        }
    }

    fn current(&self) -> u32 {
        *self.successful_saves.borrow()
    }
}

struct StatusPanel {
    status_text: Rc<RefCell<String>>,
    _subscription: SubscriptionGuard<BoxedSubscription>,
}

impl StatusPanel {
    fn new(bus: rxrust_event_bus::EventBus<AppEvent>) -> Self {
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
    let bus = rxrust_event_bus::EventBus::<AppEvent>::new();
    let stats = SaveStats::new(bus.clone());
    let status_panel = StatusPanel::new(bus.clone());

    bus.publish(AppEvent::SaveRequested);
    status_panel.render();
    bus.publish(AppEvent::SaveFinished { ok: true });
    status_panel.render();
    bus.publish(AppEvent::SaveFinished { ok: false });
    status_panel.render();
    bus.publish(AppEvent::SaveFinished { ok: true });

    assert_eq!(stats.current(), 2);
}
