// Example RUX component

fn App() -> Element {
    <div>
        <h1>Hello, RUX!</h1>
        <p>Welcome to the RUX framework</p>
    </div>
}

fn Counter() -> Element {
    let (count, set_count) = use_state(|| 0);
    
    <div>
        <p>Count: {count}</p>
        <button on_click={|| set_count(count + 1)}>
            Increment
        </button>
    </div>
}

fn Greeting(props: GreetingProps) -> Element {
    <div>
        <h1>Hello, {props.name}!</h1>
    </div>
}

struct GreetingProps {
    name: String,
}
