# RUX Component System Design

## Overview

RUX uses a functional component model inspired by React, with type-safe props, hooks for state management, and a powerful composition system. Components are pure functions that return `Element` trees.

## 1. Functional Component Model

### 1.1 Component Definition

Components are Rust functions that return `Element`.

```rsx
fn Greeting(name: String) -> Element {
    <div>
        <h1>Hello, {name}!</h1>
    </div>
}
```

### 1.2 Component Invocation

Components are invoked using JSX-like syntax.

```rsx
<Greeting name="RUX" />
```

### 1.3 Component Composition

Components can compose other components.

```rsx
fn App() -> Element {
    <div>
        <Header />
        <Main>
            <Greeting name="World" />
        </Main>
        <Footer />
    </div>
}
```

## 2. Props System

### 2.1 Props Struct

Props are defined as Rust structs with derive macros.

```rsx
#[derive(Props, Clone, PartialEq)]
struct ButtonProps {
    label: String,
    disabled: Option<bool>,
    on_click: Option<Callback<()>>,
    class: Option<String>,
}

fn Button(props: ButtonProps) -> Element {
    <button 
        class={props.class.unwrap_or_default()}
        disabled={props.disabled.unwrap_or(false)}
        on_click={props.on_click}
    >
        {props.label}
    </button>
}
```

### 2.2 Type-Safe Props

Props are type-checked at compile time.

```rsx
// Compiler error: missing required prop
<Button /> // Error: missing `label`

// Compiler error: wrong type
<Button label={42} /> // Error: expected String, found i32
```

### 2.3 Optional Props

Props can be optional using `Option<T>`.

```rsx
#[derive(Props)]
struct CardProps {
    title: String,
    subtitle: Option<String>,
    image: Option<String>,
}

fn Card(props: CardProps) -> Element {
    <div class="card">
        <h2>{props.title}</h2>
        {if let Some(subtitle) = props.subtitle {
            <h3>{subtitle}</h3>
        }}
        {if let Some(image) = props.image {
            <img src={image} />
        }}
    </div>
}
```

### 2.4 Default Props

Default values for props using `Default` trait.

```rsx
#[derive(Props)]
struct InputProps {
    #[prop(default = "text")]
    input_type: String,
    #[prop(default = false)]
    required: bool,
    value: String,
}
```

### 2.5 Props Validation

Runtime validation for props.

```rsx
fn validate_props(props: &ButtonProps) -> Result<(), String> {
    if props.label.is_empty() {
        return Err("Label cannot be empty".to_string());
    }
    Ok(())
}
```

## 3. Children Composition

### 3.1 Children Prop

Components can accept children as a prop.

```rsx
fn Container(children: Element) -> Element {
    <div class="container">
        {children}
    </div>
}

// Usage
<Container>
    <p>Child content</p>
    <button>Action</button>
</Container>
```

### 3.2 Multiple Children

Components can accept multiple named children.

```rsx
fn Layout(header: Element, main: Element, footer: Element) -> Element {
    <div class="layout">
        <header>{header}</header>
        <main>{main}</main>
        <footer>{footer}</footer>
    </div>
}

// Usage
<Layout 
    header={<Header />}
    main={<Main />}
    footer={<Footer />}
/>
```

### 3.3 Render Props

Functions as children for flexible composition.

```rsx
fn DataProvider<T>(data: T, children: fn(&T) -> Element) -> Element {
    children(&data)
}

// Usage
<DataProvider data={users} children={|users| {
    <UserList users={users} />
}} />
```

## 4. Component Lifecycle

### 4.1 Mount and Unmount

Lifecycle hooks for component mounting and unmounting.

```rsx
fn MyComponent() -> Element {
    use_effect(|| {
        // Component mounted
        println!("Component mounted");
        
        // Cleanup function
        || {
            println!("Component unmounted");
        }
    });
    
    <div>Content</div>
}
```

### 4.2 Update Lifecycle

Hooks for tracking updates.

```rsx
fn Component(props: Props) -> Element {
    use_effect_with_deps(|| {
        // Runs when props change
        println!("Props updated: {:?}", props);
    }, [props]);
    
    <div>{props.value}</div>
}
```

### 4.3 Lifecycle Phases

1. **Initialization**: Component function called
2. **Mounting**: Component added to tree
3. **Updating**: Props or state change
4. **Unmounting**: Component removed from tree

## 5. Hooks System

### 5.1 useState Hook

State management hook.

```rsx
fn Counter() -> Element {
    let (count, set_count) = use_state(|| 0);
    
    <div>
        <p>Count: {count}</p>
        <button on_click={|| set_count(count + 1)}>
            Increment
        </button>
    </div>
}
```

### 5.2 useEffect Hook

Side effects hook.

```rsx
fn DataFetcher() -> Element {
    let (data, set_data) = use_state(|| None);
    
    use_effect(|| {
        // Fetch data on mount
        let fetched = fetch_data().await;
        set_data(Some(fetched));
    });
    
    match data {
        Some(d) => <DataView data={d} />,
        None => <Loading />,
    }
}
```

### 5.3 useMemo Hook

Memoization hook for expensive computations.

```rsx
fn ExpensiveComponent(items: Vec<Item>) -> Element {
    let filtered = use_memo(|| {
        items.iter()
            .filter(|item| item.active)
            .collect::<Vec<_>>()
    }, [items]);
    
    <ItemList items={filtered} />
}
```

### 5.4 useCallback Hook

Callback memoization.

```rsx
fn Parent() -> Element {
    let (count, set_count) = use_state(|| 0);
    
    let handle_click = use_callback(|| {
        set_count(count + 1);
    }, [count]);
    
    <Child on_click={handle_click} />
}
```

### 5.5 useRef Hook

Mutable reference hook.

```rsx
fn Input() -> Element {
    let input_ref = use_ref(|| None);
    
    <input 
        ref={input_ref}
        on_focus={|| {
            if let Some(ref input) = input_ref.current() {
                input.focus();
            }
        }}
    />
}
```

### 5.6 Custom Hooks

Creating reusable hook logic.

```rsx
fn use_local_storage<T>(key: String, default: T) -> (T, fn(T)) 
where
    T: Clone + Serialize + DeserializeOwned,
{
    let (value, set_value) = use_state(|| {
        // Load from localStorage
        load_from_storage(&key).unwrap_or(default)
    });
    
    let update = |new_value: T| {
        save_to_storage(&key, &new_value);
        set_value(new_value);
    };
    
    (value, update)
}
```

## 6. Context API

### 6.1 Context Creation

Creating context for sharing data.

```rsx
struct ThemeContext {
    theme: Theme,
    toggle_theme: Callback<()>,
}

let ThemeContext = create_context::<ThemeContext>();
```

### 6.2 Context Provider

Providing context to children.

```rsx
fn App() -> Element {
    let (theme, set_theme) = use_state(|| Theme::Light);
    
    <ThemeContext.Provider value={ThemeContext {
        theme: theme.clone(),
        toggle_theme: || set_theme(theme.toggle()),
    }}>
        <Main />
    </ThemeContext.Provider>
}
```

### 6.3 Context Consumption

Consuming context in components.

```rsx
fn ThemedButton() -> Element {
    let theme = use_context::<ThemeContext>();
    
    <button class={theme.theme.button_class()}>
        Toggle Theme
    </button>
}
```

### 6.4 Multiple Contexts

Components can consume multiple contexts.

```rsx
fn Component() -> Element {
    let theme = use_context::<ThemeContext>();
    let user = use_context::<UserContext>();
    let settings = use_context::<SettingsContext>();
    
    // Use all contexts
}
```

## 7. Scoped Effects

### 7.1 Effect Scope

Effects are scoped to component lifecycle.

```rsx
fn Component() -> Element {
    use_effect(|| {
        let timer = set_interval(|| {
            println!("Tick");
        }, 1000);
        
        // Cleanup on unmount
        || {
            clear_interval(timer);
        }
    });
    
    <div>Component</div>
}
```

### 7.2 Effect Dependencies

Effects can depend on specific values.

```rsx
fn Component(id: u64) -> Element {
    use_effect_with_deps(|| {
        fetch_data(id).await;
    }, [id]); // Only re-run when id changes
}
```

### 7.3 Async Effects

Handling async operations in effects.

```rsx
fn Component() -> Element {
    let (data, set_data) = use_state(|| None);
    
    use_effect(|| {
        spawn(async move {
            let result = fetch_data().await;
            set_data(Some(result));
        });
    });
    
    // Render
}
```

## 8. Component Patterns

### 8.1 Higher-Order Components

Functions that return components.

```rsx
fn with_auth<T>(component: fn(T) -> Element) -> fn(T) -> Element {
    |props: T| {
        let user = use_context::<UserContext>();
        if user.is_authenticated() {
            component(props)
        } else {
            <LoginRequired />
        }
    }
}
```

### 8.2 Compound Components

Components that work together.

```rsx
fn Tabs() -> Element {
    let (active, set_active) = use_state(|| 0);
    
    <TabsContext.Provider value={TabsContext { active, set_active }}>
        <div class="tabs">
            {/* Tab headers and panels */}
        </div>
    </TabsContext.Provider>
}

fn TabHeader(index: u32, label: String) -> Element {
    let context = use_context::<TabsContext>();
    <div 
        class={if context.active == index { "active" } else { "" }}
        on_click={|| context.set_active(index)}
    >
        {label}
    </div>
}
```

### 8.3 Render Props Pattern

Components that accept render functions.

```rsx
fn DataProvider<T>(data: T, render: fn(&T) -> Element) -> Element {
    render(&data)
}
```

## 9. Component Performance

### 9.1 Memoization

Memoizing components to prevent unnecessary re-renders.

```rsx
#[memo]
fn ExpensiveComponent(props: Props) -> Element {
    // Only re-renders when props change
}
```

### 9.2 Lazy Loading

Lazy loading components.

```rsx
let LazyComponent = lazy(|| {
    import("./HeavyComponent.rsx")
});

<LazyComponent fallback={<Loading />} />
```

## 10. Component Testing

### 10.1 Testing Utilities

Utilities for testing components.

```rsx
#[test]
fn test_button_click() {
    let mut component = render(<Button label="Click" />);
    component.click();
    assert_eq!(component.text(), "Clicked");
}
```

## 11. Future Considerations

- Server components
- Streaming components
- Suspense boundaries
- Error boundaries
- Component code splitting
- Progressive hydration

