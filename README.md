# RX Event Bus

A simple, agnostic reactive extensions event bus utilising [rxRust](https://github.com/rxRust/rxRust) under the hood.

- Events are just a generic user-supplied type - implement however you wish
- Validate and reject invalid events
- Validation is a generic user-defined function - implement according to the event design
- Single-threaded `LocalEventBus` and thread-safe `SharedEventBus` variants using the equivalent rxRust `Local` and `Shared` contexts
- No channels! Events are published to an rxRust Observable which can be filtered accordingly (or apply any other rxRust operator)

*Note: this library uses rxRust v1.0.0-rc3. This has much better reactive extension coverage relative to the latest stable version v0.15.0 published in 2021.*

## Single-threaded example

```rust
use rx_event_bus::LocalEventBus;
use rxrust::Observable;

#[derive(Clone, Debug)]
enum ChatEvent {
    Message { user: String, text: String },
    UserJoined { user: String },
}

#[derive(Debug)]
enum ChatError {
    EmptyMessage,
    EmptyUsername,
}

fn validate(event: ChatEvent) -> Result<ChatEvent, ChatError> {
    match &event {
        ChatEvent::Message { user, text } => {
            if user.is_empty() {
                return Err(ChatError::EmptyUsername);
            }
            if text.is_empty() {
                return Err(ChatError::EmptyMessage);
            }
        }
        ChatEvent::UserJoined { user } => {
            if user.is_empty() {
                return Err(ChatError::EmptyUsername);
            }
        }
    }
    Ok(event)
}

fn main() {
    let bus = LocalEventBus::new(validate);

    // Subscribe to all events
    bus.subscribe(|event| {
        println!("  [log] {:?}", event);
    });

    // Subscribe to the event stream with filtering (only messages)
    bus.events()
        .filter(|e| matches!(e, ChatEvent::Message { .. }))
        .subscribe(|event| {
            if let ChatEvent::Message { user, text } = event {
                println!("  [chat] {user}: {text}");
            }
        });

    // Publish some events
    println!("Publishing UserJoined:");
    bus.publish(ChatEvent::UserJoined {
        user: "Alice".into(),
    })
    .unwrap();

    println!("Publishing Message:");
    bus.publish(ChatEvent::Message {
        user: "Alice".into(),
        text: "Hello, world!".into(),
    })
    .unwrap();

    // Validation rejects invalid events
    println!("Publishing invalid (empty message):");
    if let Err(e) = bus.publish(ChatEvent::Message {
        user: "Alice".into(),
        text: "".into(),
    }) {
        println!("  Rejected: {:?}", e);
    }
}
```

## Multi-threaded example

```rust
use std::sync::{Arc, Mutex};

use rx_event_bus::SharedEventBus;
use rxrust::Observable;

#[derive(Clone, Debug)]
enum ChatEvent {
    Message { user: String, text: String },
    UserJoined { user: String },
}

#[derive(Clone, Debug)]
enum ChatError {
    EmptyMessage,
    EmptyUsername,
}

fn validate(event: ChatEvent) -> Result<ChatEvent, ChatError> {
    match &event {
        ChatEvent::Message { user, text } => {
            if user.is_empty() {
                return Err(ChatError::EmptyUsername);
            }
            if text.is_empty() {
                return Err(ChatError::EmptyMessage);
            }
        }
        ChatEvent::UserJoined { user } => {
            if user.is_empty() {
                return Err(ChatError::EmptyUsername);
            }
        }
    }
    Ok(event)
}

fn main() {
    let bus = SharedEventBus::new(validate);

    // Collect events from a subscriber thread
    let events: Arc<Mutex<Vec<ChatEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_clone = events.clone();

    // Subscribe to all events
    let _sub = bus.subscribe(move |event: ChatEvent| {
        println!("  [log] {:?}", event);
        events_clone.lock().unwrap().push(event);
    });

    // Subscribe to the event stream with filtering (only messages)
    let _sub2 = bus
        .events()
        .filter(|e| matches!(e, ChatEvent::Message { .. }))
        .subscribe(move |event| {
            if let ChatEvent::Message { user, text } = event {
                println!("  [chat] {user}: {text}");
            }
        });

    // Publish from the main thread
    println!("Publishing UserJoined:");
    bus.publish(ChatEvent::UserJoined {
        user: "Alice".into(),
    })
    .unwrap();

    // Publish from a spawned thread
    let bus_clone = bus.clone();
    let handle = std::thread::spawn(move || {
        println!("Publishing Message from another thread:");
        bus_clone
            .publish(ChatEvent::Message {
                user: "Bob".into(),
                text: "Hello from another thread!".into(),
            })
            .unwrap();
    });
    handle.join().unwrap();

    // Validation rejects invalid events
    println!("Publishing invalid (empty message):");
    if let Err(e) = bus.publish(ChatEvent::Message {
        user: "Alice".into(),
        text: "".into(),
    }) {
        println!("  Rejected: {:?}", e);
    }

    // Show collected events
    let collected = events.lock().unwrap();
    println!("\nCollected {} events total.", collected.len());
}
```
