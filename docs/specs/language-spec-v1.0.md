# RUX Language Specification v1.0

## Overview

RUX is a Rust-based declarative UI language that combines the best features from modern UI frameworks with Rust's safety guarantees and zero-cost abstractions. This specification defines the core language features, type system, and semantics.

## 1. Type System

### 1.1 Strong Static Typing

RUX enforces strong static typing at compile time. All values must have explicit or inferred types.

```rsx
// Explicit type annotation
let count: i32 = 0;

// Type inference
let name = "RUX"; // inferred as &str
let items = vec![1, 2, 3]; // inferred as Vec<i32>
```

### 1.2 Type Inference

RUX uses Rust's powerful type inference system. Types can be omitted when the compiler can infer them from context.

```rsx
// Inference from function return type
fn get_count() -> i32 { 42 }
let count = get_count(); // inferred as i32

// Inference from usage
let mut items = Vec::new(); // type must be specified or inferred from push
items.push(1); // now inferred as Vec<i32>
```

### 1.3 Generics and Variance

RUX supports Rust-style generics with variance rules for type safety.

```rsx
// Generic component
fn List<T>(items: Vec<T>, render: fn(&T) -> Element) -> Element {
    // ...
}

// Variance: covariant, contravariant, invariant
struct Container<T> {
    value: T,
}
```

### 1.4 Algebraic Data Types (ADTs)

RUX supports sum types (enums) and product types (structs) as first-class citizens.

```rsx
// Sum type (enum)
enum Message {
    Text(String),
    Image { url: String, alt: String },
    Empty,
}

// Product type (struct)
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}
```

### 1.5 Enums with Data

Enums can carry associated data, enabling pattern matching and exhaustive checking.

```rsx
enum State<T> {
    Loading,
    Loaded(T),
    Error(String),
}

// Pattern matching
match state {
    State::Loading => <Spinner />,
    State::Loaded(data) => <DataView data={data} />,
    State::Error(msg) => <Error message={msg} />,
}
```

## 2. Ownership and Borrowing

### 2.1 Ownership Model

RUX uses Rust's ownership system to ensure memory safety without garbage collection.

```rsx
// Ownership transfer
fn take_ownership(s: String) {
    // s is now owned by this function
}

let my_string = String::from("hello");
take_ownership(my_string); // ownership moved
// my_string is no longer valid here
```

### 2.2 Borrowing for UI Components

Components can borrow props to avoid unnecessary cloning.

```rsx
// Borrowing props
fn UserCard<'a>(user: &'a User) -> Element {
    <div>
        <h1>{user.name}</h1>
        <p>{user.email.as_ref()}</p>
    </div>
}

// Ownership for state
fn Counter() -> Element {
    let count = use_state(|| 0);
    // count is owned by the component's state
}
```

### 2.3 Move Semantics

RUX supports zero-cost moves, enabling efficient component composition.

```rsx
// Move semantics for children
fn Container(children: Element) -> Element {
    <div class="container">
        {children} // moved, not cloned
    </div>
}
```

## 3. Pattern Matching

### 3.1 Exhaustive Pattern Matching

Pattern matching must be exhaustive, ensuring all cases are handled.

```rsx
enum Status {
    Pending,
    Success,
    Failed,
}

match status {
    Status::Pending => <Spinner />,
    Status::Success => <SuccessIcon />,
    Status::Failed => <ErrorIcon />,
    // Compiler error if any case is missing
}
```

### 3.2 Destructuring

Pattern matching supports destructuring of structs and tuples.

```rsx
struct Point { x: f64, y: f64 }

match point {
    Point { x, y } if x > 0.0 && y > 0.0 => <Quadrant1 />,
    Point { x: 0.0, y } => <YAxis />,
    Point { x, y: 0.0 } => <XAxis />,
    _ => <Origin />,
}
```

### 3.3 Guards

Pattern guards allow additional conditions in match expressions.

```rsx
match value {
    x if x > 100 => <HighValue />,
    x if x > 50 => <MediumValue />,
    _ => <LowValue />,
}
```

## 4. Traits and Interfaces

### 4.1 Trait Definitions

Traits define shared behavior that types can implement.

```rsx
trait Renderable {
    fn render(&self) -> Element;
}

struct Button {
    label: String,
}

impl Renderable for Button {
    fn render(&self) -> Element {
        <button>{self.label.clone()}</button>
    }
}
```

### 4.2 Extension Methods

Traits can extend existing types with new methods.

```rsx
trait StringExt {
    fn to_title_case(&self) -> String;
}

impl StringExt for String {
    fn to_title_case(&self) -> String {
        // implementation
    }
}

// Usage
let title = "hello world".to_string().to_title_case();
```

### 4.3 Trait Bounds

Generic functions can constrain types with trait bounds.

```rsx
fn render_item<T: Renderable>(item: &T) -> Element {
    item.render()
}
```

## 5. Null Safety

### 5.1 Option Type

RUX uses `Option<T>` instead of null, ensuring null-safety at compile time.

```rsx
// No null, only Option
let maybe_name: Option<String> = Some("RUX".to_string());
let empty: Option<String> = None;

// Safe unwrapping
match maybe_name {
    Some(name) => <div>{name}</div>,
    None => <div>No name</div>,
}
```

### 5.2 Result Type

Error handling uses `Result<T, E>` instead of exceptions.

```rsx
fn fetch_data() -> Result<String, String> {
    // returns Ok(value) or Err(error)
}

match fetch_data() {
    Ok(data) => <DataView data={data} />,
    Err(error) => <Error message={error} />,
}
```

## 6. Data Classes and Structs

### 6.1 Struct Definitions

Structs define product types with named fields.

```rsx
#[derive(Clone, Debug, PartialEq)]
struct User {
    id: u64,
    name: String,
    email: String,
    age: Option<u32>,
}
```

### 6.2 Tuple Structs

Tuple structs provide unnamed field access.

```rsx
struct Point(f64, f64);
let origin = Point(0.0, 0.0);
```

### 6.3 Unit Structs

Unit structs represent types without data.

```rsx
struct Marker;
```

## 7. Macro System

### 7.1 Declarative Macros

RUX supports Rust's `macro_rules!` for DSL creation.

```rsx
macro_rules! component {
    ($name:ident($($prop:ident: $type:ty),*) -> $body:expr) => {
        fn $name($($prop: $type),*) -> Element {
            $body
        }
    };
}

// Usage
component!(Greeting(name: String) -> <div>Hello, {name}!</div>);
```

### 7.2 Procedural Macros

Procedural macros enable advanced code generation for components.

```rsx
#[component]
fn MyComponent(props: Props) -> Element {
    // macro generates component boilerplate
}
```

### 7.3 DSL Macros

Macros enable domain-specific languages for UI patterns.

```rsx
// Example: Animation DSL
animate! {
    duration: 300ms,
    easing: ease-in-out,
    from: { opacity: 0, transform: translateY(10px) },
    to: { opacity: 1, transform: translateY(0) },
}
```

## 8. Type System Guarantees

### 8.1 Compile-Time Safety

All type errors are caught at compile time, preventing runtime type errors.

### 8.2 Zero-Cost Abstractions

Type system features compile to efficient code with no runtime overhead.

### 8.3 Lifetime Safety

The borrow checker ensures memory safety without runtime checks.

## 9. Future Considerations

- Higher-kinded types (HKT)
- Specialization for performance-critical paths
- Const generics for compile-time computations
- Associated type constructors
- GATs (Generic Associated Types)

## 10. Language Goals

1. **Safety**: Memory safety and type safety guaranteed at compile time
2. **Performance**: Zero-cost abstractions, no runtime overhead
3. **Expressiveness**: Rich type system enabling powerful abstractions
4. **Ergonomics**: Type inference and macros reduce boilerplate
5. **Interoperability**: Seamless integration with Rust ecosystem

