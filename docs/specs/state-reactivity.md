# RUX State & Reactivity Design

## Overview

RUX combines the best reactivity models from modern frameworks: fine-grained signals (SolidJS), reactive stores (Svelte), reducer-based state (Redux/Elm), and observables (RxJS). This creates a powerful, performant state management system.

## 1. Fine-Grained Signals

### 1.1 Signal Creation

Signals are the foundation of RUX reactivity, providing O(1) updates.

```rsx
fn Component() -> Element {
    let count = use_signal(|| 0);
    let name = use_signal(|| "RUX".to_string());
    
    <div>
        <p>Count: {count()}</p>
        <p>Name: {name()}</p>
    </div>
}
```

### 1.2 Signal Updates

Signals can be read and written.

```rsx
let count = use_signal(|| 0);

// Read
let current = count();

// Write
count.set(42);

// Update
count.update(|n| *n + 1);
```

### 1.3 Derived Signals

Signals can derive from other signals.

```rsx
let first = use_signal(|| 10);
let second = use_signal(|| 20);

let sum = create_memo(|| first() + second());

<div>Sum: {sum()}</div>
```

### 1.4 Signal Granularity

Only the specific parts that depend on a signal update, not entire components.

```rsx
let count = use_signal(|| 0);

<div>
    <p>Static text</p> {/* Never re-renders */}
    <p>Count: {count()}</p> {/* Only this updates */}
    <p>More static text</p> {/* Never re-renders */}
</div>
```

## 2. Reactive Stores

### 2.1 Store Definition

Stores provide reactive state management.

```rsx
#[derive(Clone)]
struct UserStore {
    name: String,
    email: String,
    age: u32,
}

let user_store = create_store(UserStore {
    name: "John".to_string(),
    email: "john@example.com".to_string(),
    age: 30,
});
```

### 2.2 Store Subscription

Components automatically subscribe to store changes.

```rsx
fn UserProfile() -> Element {
    let user = use_store(user_store);
    
    <div>
        <h1>{user().name}</h1>
        <p>{user().email}</p>
    </div>
}
```

### 2.3 Store Updates

Updating stores triggers reactive updates.

```rsx
// Update entire store
user_store.set(UserStore {
    name: "Jane".to_string(),
    email: "jane@example.com".to_string(),
    age: 25,
});

// Partial update
user_store.update(|user| {
    user.name = "Jane".to_string();
});
```

### 2.4 Reactive Store Syntax

Svelte-style reactive syntax.

```rsx
let user = user_store();

// Automatically reactive
$: full_name = format!("{} {}", user.name, user.email);
```

## 3. Auto Dependency Tracking

### 3.1 Automatic Tracking

Dependencies are automatically tracked in reactive contexts.

```rsx
let a = use_signal(|| 1);
let b = use_signal(|| 2);

// Automatically tracks a and b
let result = create_memo(|| {
    a() * b() // Tracks both signals
});
```

### 3.2 Dependency Graph

RUX builds a dependency graph for efficient updates.

```rsx
let input = use_signal(|| 0);
let doubled = create_memo(|| input() * 2);
let quadrupled = create_memo(|| doubled() * 2);

// Updating input automatically updates doubled and quadrupled
```

### 3.3 Batch Updates

Multiple updates can be batched.

```rsx
batch(|| {
    count.set(10);
    name.set("RUX".to_string());
    active.set(true);
    // All updates happen together, single re-render
});
```

## 4. Reducer-Based Global Store

### 4.1 Reducer Definition

Pure reducer functions for predictable state transitions.

```rsx
enum CounterAction {
    Increment,
    Decrement,
    Set(i32),
}

fn counter_reducer(state: &i32, action: CounterAction) -> i32 {
    match action {
        CounterAction::Increment => state + 1,
        CounterAction::Decrement => state - 1,
        CounterAction::Set(value) => value,
    }
}
```

### 4.2 Store Creation

Creating a reducer-based store.

```rsx
let counter_store = create_reducer_store(0, counter_reducer);
```

### 4.3 Dispatching Actions

Dispatching actions to update state.

```rsx
fn Counter() -> Element {
    let (state, dispatch) = use_reducer_store(counter_store);
    
    <div>
        <p>Count: {state}</p>
        <button on_click={|| dispatch(CounterAction::Increment)}>
            +
        </button>
        <button on_click={|| dispatch(CounterAction::Decrement)}>
            -
        </button>
    </div>
}
```

### 4.4 Async Actions

Handling async operations in reducers.

```rsx
enum DataAction {
    Load,
    LoadSuccess(Vec<Item>),
    LoadError(String),
}

async fn data_reducer(state: &DataState, action: DataAction) -> DataState {
    match action {
        DataAction::Load => {
            // Start loading
            spawn(async move {
                match fetch_data().await {
                    Ok(data) => dispatch(DataAction::LoadSuccess(data)),
                    Err(e) => dispatch(DataAction::LoadError(e.to_string())),
                }
            });
            DataState::Loading
        }
        DataAction::LoadSuccess(items) => DataState::Loaded(items),
        DataAction::LoadError(error) => DataState::Error(error),
    }
}
```

### 4.5 Time-Travel Debugging

Redux DevTools-style time-travel debugging.

```rsx
let store = create_reducer_store_with_devtools(
    0,
    counter_reducer,
    DevToolsConfig::default(),
);
```

## 5. Observable Streams

### 5.1 Observable Creation

RxJS-style observables for stream-based updates.

```rsx
let click_stream = create_observable(|observer| {
    let handler = move |event| {
        observer.next(event);
    };
    // Setup event listener
    handler
});
```

### 5.2 Observable Operators

Common operators for transforming streams.

```rsx
let debounced_clicks = click_stream
    .debounce(300)
    .map(|event| event.target())
    .filter(|target| target.is_button());
```

### 5.3 Observable Subscription

Subscribing to observables in components.

```rsx
fn Component() -> Element {
    let (clicks, set_clicks) = use_state(|| 0);
    
    use_observable(click_stream, |event| {
        set_clicks(clicks + 1);
    });
    
    <div>Clicks: {clicks}</div>
}
```

### 5.4 Combining Observables

Combining multiple observables.

```rsx
let combined = combine_latest(
    user_stream,
    settings_stream,
    |user, settings| {
        (user.clone(), settings.clone())
    }
);
```

## 6. State-Driven Recomposition

### 6.1 Reactive Recomposition

Components recompose only when their dependencies change.

```rsx
fn Component() -> Element {
    let count = use_signal(|| 0);
    let name = use_signal(|| "RUX".to_string());
    
    // Only recomposes when count changes
    let doubled = create_memo(|| count() * 2);
    
    <div>
        <p>{doubled()}</p>
        {/* name changes don't trigger recomposition here */}
    </div>
}
```

### 6.2 Selective Updates

Only affected parts of the UI update.

```rsx
let items = use_signal(|| vec![1, 2, 3]);

<div>
    {#for item in items()}
        <Item key={item} value={item} />
        {/* Only changed items update */}
    {/for}
</div>
```

## 7. State Composition

### 7.1 Combining State Sources

Combining multiple state sources.

```rsx
fn Component() -> Element {
    let local_state = use_signal(|| 0);
    let global_state = use_store(global_store);
    let context_state = use_context::<AppContext>();
    
    // Combine all sources
    let combined = create_memo(|| {
        (local_state(), global_state(), context_state.clone())
    });
    
    // Use combined state
}
```

### 7.2 State Normalization

Normalizing complex state structures.

```rsx
struct NormalizedState {
    users: HashMap<u64, User>,
    posts: HashMap<u64, Post>,
    comments: HashMap<u64, Comment>,
}
```

## 8. State Persistence

### 8.1 Local Storage

Persisting state to local storage.

```rsx
let persisted_state = create_persisted_signal(
    "key",
    || 0,
    |value| serde_json::to_string(value).unwrap(),
    |s| serde_json::from_str(s).unwrap(),
);
```

### 8.2 Session Storage

Session-based persistence.

```rsx
let session_state = create_session_signal("key", || default_value);
```

### 8.3 IndexedDB

Complex data persistence.

```rsx
let db_state = create_indexeddb_signal("database", "store", || default_value);
```

## 9. State Synchronization

### 9.1 Server Synchronization

Syncing state with server.

```rsx
let synced_state = create_synced_signal(
    "api/endpoint",
    || default_value,
    SyncConfig {
        debounce: 300,
        on_conflict: ConflictResolution::ServerWins,
    },
);
```

### 9.2 Real-Time Updates

WebSocket-based real-time state.

```rsx
let realtime_state = create_realtime_signal(
    "ws://server/updates",
    || default_value,
);
```

## 10. Performance Optimizations

### 10.1 Memoization

Memoizing expensive computations.

```rsx
let expensive = create_memo(|| {
    // Expensive computation
    items.iter()
        .filter(|item| item.active)
        .map(|item| process(item))
        .collect::<Vec<_>>()
});
```

### 10.2 Lazy Evaluation

Lazy evaluation of reactive values.

```rsx
let lazy_value = create_lazy_signal(|| {
    // Only computed when accessed
    expensive_computation()
});
```

### 10.3 Update Batching

Batching multiple updates.

```rsx
batch(|| {
    state1.set(value1);
    state2.set(value2);
    state3.set(value3);
    // Single update cycle
});
```

## 11. State Testing

### 11.1 Testing Signals

Testing signal behavior.

```rsx
#[test]
fn test_signal() {
    let signal = create_signal(0);
    assert_eq!(signal(), 0);
    signal.set(42);
    assert_eq!(signal(), 42);
}
```

### 11.2 Testing Reducers

Testing reducer functions.

```rsx
#[test]
fn test_reducer() {
    let state = 0;
    let new_state = counter_reducer(&state, CounterAction::Increment);
    assert_eq!(new_state, 1);
}
```

## 12. Future Considerations

- State machines (XState-style)
- GraphQL integration
- Optimistic updates
- Conflict resolution strategies
- State versioning
- State migration

