# RUX Syntax Manual

## Overview

RUX uses a JSX-like syntax with Rust semantics, combined with SwiftUI-style modifiers and Svelte-style reactivity. This manual documents all syntax features for `.rsx` files.

## 1. Basic Syntax

### 1.1 File Extension

RUX components use the `.rsx` file extension.

```rsx
// component.rsx
fn MyComponent() -> Element {
    <div>Hello, RUX!</div>
}
```

### 1.2 Component Declaration

Components are Rust functions that return `Element`.

```rsx
fn Button() -> Element {
    <button>Click me</button>
}
```

### 1.3 Self-Closing Tags

Tags can be self-closing when they have no children.

```rsx
<img src="logo.png" alt="Logo" />
<br />
<input type="text" />
```

## 2. JSX-Like Tag Syntax

### 2.1 HTML-Like Elements

RUX supports HTML-like element syntax.

```rsx
<div>
    <h1>Title</h1>
    <p>Paragraph text</p>
</div>
```

### 2.2 Component Tags

Custom components use PascalCase and can be used like HTML elements.

```rsx
<Header />
<UserProfile user={current_user} />
<Navigation>
    <NavItem href="/home">Home</NavItem>
    <NavItem href="/about">About</NavItem>
</Navigation>
```

### 2.3 Fragment Syntax

Fragments allow multiple elements without a wrapper.

```rsx
<>
    <Header />
    <Main />
    <Footer />
</>
```

## 3. Props and Attributes

### 3.1 String Attributes

String attributes use quotes.

```rsx
<div class="container" id="main">
    Content
</div>
```

### 3.2 Expression Attributes

Expressions use curly braces.

```rsx
<div class={dynamic_class} style={styles}>
    <img src={image_url} alt={alt_text} />
</div>
```

### 3.3 Boolean Attributes

Boolean attributes can be shorthand.

```rsx
<input type="checkbox" checked={is_checked} />
<input type="checkbox" disabled />
```

### 3.4 Spread Attributes

Props can be spread from a struct.

```rsx
let props = ButtonProps {
    label: "Click me".to_string(),
    disabled: false,
    class: "primary".to_string(),
};

<Button {...props} />
```

## 4. Children and Composition

### 4.1 Text Children

Text can be direct children.

```rsx
<button>Click me</button>
<h1>Welcome to RUX</h1>
```

### 4.2 Element Children

Elements can be nested.

```rsx
<div>
    <h1>Title</h1>
    <p>Content</p>
</div>
```

### 4.3 Expression Children

Children can be expressions.

```rsx
<div>
    {if condition {
        <SuccessMessage />
    } else {
        <ErrorMessage />
    }}
</div>
```

### 4.4 Children Prop

Components can accept children explicitly.

```rsx
fn Container(children: Element) -> Element {
    <div class="container">
        {children}
    </div>
}

// Usage
<Container>
    <p>Child content</p>
</Container>
```

## 5. Inline Expressions

### 5.1 Interpolation

Expressions are interpolated with curly braces.

```rsx
<div>
    <h1>Hello, {user.name}!</h1>
    <p>You have {count} items</p>
</div>
```

### 5.2 Conditional Rendering

Conditional expressions for rendering.

```rsx
<div>
    {if is_logged_in {
        <UserMenu />
    } else {
        <LoginButton />
    }}
</div>
```

### 5.3 Ternary Expressions

Ternary-style conditionals.

```rsx
<div>
    {if count > 0 { count.to_string() } else { "No items".to_string() }}
</div>
```

### 5.4 Match Expressions

Pattern matching in templates.

```rsx
<div>
    {match status {
        Status::Loading => <Spinner />,
        Status::Success => <SuccessIcon />,
        Status::Error => <ErrorIcon />,
    }}
</div>
```

## 6. Modifier Chains

### 6.1 SwiftUI-Style Modifiers

Modifiers chain using dot notation.

```rsx
<Text>
    "Hello, World!"
</Text>
    .padding(16)
    .background(Color::blue())
    .foreground_color(Color::white())
    .font_size(18)
    .font_weight(FontWeight::Bold)
```

### 6.2 Flutter-Style Modifiers

Alternative modifier syntax.

```rsx
<Container>
    <Text>Hello</Text>
</Container>
    .padding(all: 16)
    .margin(top: 8, bottom: 8)
    .border(width: 1, color: Color::gray())
```

### 6.3 Chaining Order

Modifiers are applied in order.

```rsx
<Button>
    "Submit"
</Button>
    .padding(12)
    .background(Color::blue())
    .corner_radius(8)
    .shadow(radius: 4)
```

## 7. Directives

### 7.1 Conditional Directive (if)

Conditional rendering directive.

```rsx
<div>
    <h1>Title</h1>
    {#if show_content}
        <p>This content is conditionally shown</p>
    {/if}
</div>
```

### 7.2 Loop Directive (for)

Iteration directive.

```rsx
<ul>
    {#for item in items}
        <li key={item.id}>{item.name}</li>
    {/for}
</ul>
```

### 7.3 Keyed Lists

Keys for efficient list updates.

```rsx
{#for user in users}
    <UserCard key={user.id} user={user} />
{/for}
```

### 7.4 Index in Loops

Accessing index in loops.

```rsx
{#for (index, item) in items.iter().enumerate()}
    <Item index={index} item={item} />
{/for}
```

### 7.5 Else Blocks

Else branches for conditionals.

```rsx
{#if items.is_empty()}
    <EmptyState />
{:else}
    <ItemList items={items} />
{/if}
```

## 8. Template-Driven Reactivity

### 8.1 Reactive Expressions

Expressions automatically track dependencies.

```rsx
let count = use_signal(0);

<div>
    <p>Count: {count()}</p>
    <button on_click={|| count.set(count() + 1)}>
        Increment
    </button>
</div>
```

### 8.2 Reactive Blocks

Blocks automatically re-evaluate when dependencies change.

```rsx
let name = use_signal("RUX".to_string());

<div>
    {#if name().len() > 0}
        <h1>Hello, {name()}!</h1>
    {/if}
</div>
```

### 8.3 Computed Values

Derived reactive values.

```rsx
let first_name = use_signal("John".to_string());
let last_name = use_signal("Doe".to_string());

let full_name = create_memo(|| {
    format!("{} {}", first_name(), last_name())
});

<div>{full_name()}</div>
```

## 9. Event Handlers

### 9.1 Click Events

Click event handlers.

```rsx
<button on_click={|| {
    println!("Clicked!");
}}>
    Click me
</button>
```

### 9.2 Event Parameters

Accessing event parameters.

```rsx
<input 
    type="text"
    on_input={|event| {
        let value = event.target().value();
        handle_input(value);
    }}
/>
```

### 9.3 Event Modifiers

Event modifiers for common patterns.

```rsx
<button 
    on_click={handle_click}
    prevent_default
    stop_propagation
>
    Submit
</button>
```

## 10. Bindings

### 10.1 Two-Way Binding

Two-way data binding.

```rsx
let name = use_signal("".to_string());

<input 
    type="text"
    bind:value={name}
/>
```

### 10.2 Checkbox Binding

Checkbox bindings.

```rsx
let checked = use_signal(false);

<input 
    type="checkbox"
    bind:checked={checked}
/>
```

### 10.3 Select Binding

Select element bindings.

```rsx
let selected = use_signal(None);

<select bind:value={selected}>
    <option value="option1">Option 1</option>
    <option value="option2">Option 2</option>
</select>
```

## 11. Comments

### 11.1 HTML-Style Comments

Comments in templates.

```rsx
<div>
    <!-- This is a comment -->
    <p>Visible content</p>
</div>
```

### 11.2 Rust Comments

Rust comments in expressions.

```rsx
<div>
    {let value = 42; // Rust comment
     value}
</div>
```

## 12. Advanced Syntax

### 12.1 Dynamic Components

Dynamic component selection.

```rsx
let component_type = use_signal(ComponentType::Button);

<div>
    {match component_type() {
        ComponentType::Button => <Button />,
        ComponentType::Link => <Link />,
    }}
</div>
```

### 12.2 Slots

Named slots for component composition.

```rsx
fn Card(header: Element, body: Element, footer: Option<Element>) -> Element {
    <div class="card">
        <div class="header">{header}</div>
        <div class="body">{body}</div>
        {if let Some(f) = footer {
            <div class="footer">{f}</div>
        }}
    </div>
}

// Usage
<Card 
    header={<h2>Title</h2>}
    body={<p>Content</p>}
    footer={Some(<button>Action</button>)}
/>
```

### 12.3 Portals

Rendering to different DOM locations.

```rsx
<Portal target="#modal-root">
    <Modal>
        <p>This renders in a different location</p>
    </Modal>
</Portal>
```

## 13. Syntax Rules Summary

1. **Components**: PascalCase functions returning `Element`
2. **Elements**: lowercase HTML-like tags
3. **Props**: snake_case attributes
4. **Expressions**: `{expression}` syntax
5. **Modifiers**: Chain with dot notation
6. **Directives**: `{#if}`, `{#for}` syntax
7. **Reactivity**: Automatic dependency tracking
8. **Bindings**: `bind:prop={signal}` syntax

## 14. Future Considerations

- JSX transform options
- Custom element types
- Template literal syntax
- Pipeline operator for modifiers
- Async component syntax

