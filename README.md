# RUX

**R**ust-based **U**ser e**X**perience - A declarative UI language combining the best features from React, Flutter, SwiftUI, SolidJS, Svelte, and more.

## Overview

RUX is a modern, type-safe, multi-platform UI framework built on Rust. It combines:

- **Best syntax** from JSX/SwiftUI/Compose
- **Best state management** from React/Solid/Vue/Svelte
- **Best layout** from CSS/Flutter/SwiftUI
- **Best performance** from Solid/Svelte/Rust
- **Best compiler** from Rust/Svelte
- **Best build tools** from Vite/Remix/Next
- **Best runtime** from Flutter/React Fiber
- **Best async model** from Kotlin/JS
- **Best platform reach** (web, mobile, desktop, embedded)

## Key Features

### Language Design
- ✅ Strong static typing with type inference
- ✅ Ownership & borrowing (memory safety without GC)
- ✅ Pattern matching and ADTs
- ✅ Traits and extension methods
- ✅ Null-safety with Option/Result
- ✅ Zero-cost abstractions
- ✅ Powerful macro system for DSLs

### UI Syntax
- ✅ JSX-like syntax (`.rsx` files)
- ✅ Functional components
- ✅ Props and children composition
- ✅ Modifier chains (SwiftUI/Flutter style)
- ✅ Template-driven reactivity
- ✅ Directives (v-if, v-for equivalents)
- ✅ Inline expressions and bindings

### State & Reactivity
- ✅ Fine-grained signals (SolidJS-style)
- ✅ Hooks (useState, useEffect, useMemo, useCallback)
- ✅ Reactive stores (Svelte-style)
- ✅ Reducer-based global store (Redux-style)
- ✅ Observable streams (RxJS-style)
- ✅ Auto dependency tracking
- ✅ Time-travel debugging

### Layout System
- ✅ Flex layout (Flutter/CSS)
- ✅ Stack-based layout (VStack, HStack, ZStack)
- ✅ CSS Grid
- ✅ Constraint-based layout (Cassowary)
- ✅ GeometryReader
- ✅ SafeAreas
- ✅ Media queries / Responsive design
- ✅ LazyColumn (virtualized lists)

### Rendering
- ✅ GPU-accelerated rendering (WGPU)
- ✅ WASM for web
- ✅ Retained-mode scene graph
- ✅ Concurrent rendering (React Fiber-style)
- ✅ Low-memory embedded mode
- ✅ GPU-less fallback

### Platform Support
- ✅ **Web**: WASM, SSR, CSR, partial hydration
- ✅ **Desktop**: WGPU renderer, window management, native dialogs
- ✅ **Mobile**: Gesture engine, multi-touch, high-DPI scaling
- ✅ **Embedded**: Low-memory mode, zero-allocation rendering

### Developer Experience
- ✅ Language Server Protocol (LSP)
- ✅ IntelliSense / Autocomplete
- ✅ Real-time diagnostics
- ✅ UI Inspector
- ✅ State Inspector
- ✅ Timeline Profiler
- ✅ Hot Module Replacement (HMR)
- ✅ File-based routing

## Quick Start

### Installation

```bash
# Add RUX to your Cargo.toml
cargo add rux
```

### Hello World

```rsx
// src/main.rsx
fn App() -> Element {
    <div>
        <h1>Hello, RUX!</h1>
    </div>
}

fn main() {
    rux::mount(<App />);
}
```

### Component with State

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

### Component with Props

```rsx
#[derive(Props, Clone)]
struct GreetingProps {
    name: String,
}

fn Greeting(props: GreetingProps) -> Element {
    <div>
        <h1>Hello, {props.name}!</h1>
    </div>
}

// Usage
<Greeting name="RUX" />
```

## Documentation

### Specifications

- [Language Specification v1.0](docs/specs/language-spec-v1.0.md) - Core language features
- [Syntax Manual](docs/specs/syntax-manual.md) - `.rsx` syntax reference
- [Component System](docs/specs/component-system.md) - Components, props, hooks
- [State & Reactivity](docs/specs/state-reactivity.md) - Signals, stores, observables
- [Runtime Architecture](docs/specs/runtime-architecture.md) - Rendering, scheduling
- [Compiler Architecture](docs/specs/compiler-architecture.md) - Parser, optimizations
- [Layout System](docs/specs/layout-system.md) - Flex, stacks, grid, constraints
- [Rendering Pipeline](docs/specs/rendering-pipeline.md) - GPU, WASM, embedded
- [Platform Features](docs/specs/platform-features.md) - Web, desktop, mobile, embedded
- [Build System & Tooling](docs/specs/build-tooling.md) - HMR, bundling, dev server
- [Dev Experience](docs/specs/dev-experience.md) - LSP, IntelliSense, inspector
- [Routing System](docs/specs/routing-system.md) - File-based routing
- [Algorithms](docs/specs/algorithms.md) - Core algorithms reference

### Comparisons

- [Framework Comparison](docs/comparisons/framework-comparison.md) - RUX vs React/Flutter/SwiftUI/etc.
- [Feature Map](docs/feature-map.md) - Complete feature list with priorities

## Project Status

**Current Phase**: Specification & Design

This repository contains the comprehensive specifications and design documents for RUX. Implementation will follow these specifications.

### Roadmap

- **Phase 1**: Core Language & Compiler
- **Phase 2**: Runtime & Rendering
- **Phase 3**: Platform Implementations
- **Phase 4**: Tooling & Ecosystem

## Design Goals

1. **Safety**: Memory safety and type safety guaranteed at compile time
2. **Performance**: Zero-cost abstractions, no runtime overhead
3. **Expressiveness**: Rich type system enabling powerful abstractions
4. **Ergonomics**: Type inference and macros reduce boilerplate
5. **Multi-platform**: Single codebase for web, desktop, mobile, embedded
6. **Developer Experience**: Excellent tooling and error messages

## Inspiration

RUX draws inspiration from:

- **React**: JSX syntax, hooks, component model
- **Flutter**: GPU rendering, widget system, multi-platform
- **SwiftUI**: Modifiers, stack layout, declarative syntax
- **SolidJS**: Fine-grained reactivity, signals
- **Svelte**: Compile-time optimizations, reactive syntax
- **Vue**: Directives, computed properties
- **Jetpack Compose**: Declarative composables
- **Rust**: Memory safety, zero-cost abstractions, type system

## Contributing

Contributions are welcome! Please see our contributing guidelines (to be added).

## License

[License to be determined]

## Acknowledgments

RUX is inspired by the excellent work of the React, Flutter, SwiftUI, SolidJS, Svelte, Vue, and Jetpack Compose teams, as well as the Rust language team.

---

**Note**: RUX is currently in the specification phase. Implementation will begin after specification completion.

