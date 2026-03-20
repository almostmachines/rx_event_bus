use std::{convert::Infallible, rc::Rc};

use rxrust::{
    Local, LocalBoxedObservableClone, LocalSubject, Observable, ObservableFactory, Observer,
    Subscription,
};

type EventBusInner<E> = LocalSubject<'static, E, Infallible>;
type EventStream<E> = LocalBoxedObservableClone<'static, E, Infallible>;
type EventValidator<E, V> = Rc<dyn Fn(E) -> Result<E, V> + 'static>;

#[derive(Clone)]
pub struct EventBus<E, V> {
    inner: EventBusInner<E>,
    validate: EventValidator<E, V>,
}

impl<E: Clone + 'static, V> EventBus<E, V> {
    pub fn new<F>(validator: F) -> Self
    where
        F: Fn(E) -> Result<E, V> + 'static,
    {
        Self {
            inner: Local::subject::<E, Infallible>(),
            validate: Rc::new(validator),
        }
    }

    pub fn publish(&self, event: E) -> Result<E, V> {
        let mut subject = self.inner.clone();
        let validated = (self.validate)(event);

        if let Ok(evt) = &validated {
            subject.next(evt.clone());
        }

        validated
    }

    pub fn events(&self) -> EventStream<E> {
        self.inner.clone().box_it_clone()
    }

    pub fn subscribe<S>(&self, handler: S) -> impl Subscription + use<E, V, S>
    where
        S: FnMut(E) + 'static,
    {
        self.events().subscribe(handler)
    }
}
