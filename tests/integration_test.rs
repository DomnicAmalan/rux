use rux_compiler::{Compiler, CodeGenerator};
use rux_runtime::executor::ComponentExecutor;
use rux_core::virtual_tree::VirtualNode;

#[test]
fn test_component_compilation_and_execution() {
    let source = r#"
fn App() -> Element {
    <div>
        <h1>Hello, RUX!</h1>
    </div>
}
"#;
    
    // Compile
    let mut compiler = Compiler::new();
    let ast = compiler.compile_string(source, "test.rsx").expect("Should compile");
    
    // Generate Rust code
    let mut codegen = CodeGenerator::new();
    let rust_code = codegen.generate_rust_code(&ast).expect("Should generate code");
    
    // Verify code generation
    assert!(rust_code.contains("pub fn app"));
    assert!(rust_code.contains("VirtualNode"));
    assert!(rust_code.contains("NodeType::Element"));
}

#[test]
fn test_jsx_to_virtual_tree() {
    let source = r#"
fn Counter() -> Element {
    <div>
        <p>Count: 0</p>
    </div>
}
"#;
    
    let mut compiler = Compiler::new();
    let ast = compiler.compile_string(source, "test.rsx").expect("Should compile");
    
    // Find the component
    let component = ast.items.iter().find_map(|item| {
        if let rux_compiler::ast::Item::Component(c) = item {
            Some(c)
        } else {
            None
        }
    }).expect("Should have component");
    
    // Execute component
    let executor = ComponentExecutor::new();
    let virtual_node = executor.execute_component(component);
    
    // Verify virtual tree structure
    match virtual_node.node_type {
        rux_core::virtual_tree::NodeType::Element(tag) => {
            assert_eq!(tag, "div");
        }
        _ => panic!("Expected Element node"),
    }
    
    assert!(!virtual_node.children.is_empty());
}

#[test]
fn test_build_system_compilation() {
    // Test that we can compile multiple files
    let source1 = r#"
fn Component1() -> Element {
    <div>Component 1</div>
}
"#;
    
    let source2 = r#"
fn Component2() -> Element {
    <div>Component 2</div>
}
"#;
    
    let mut compiler = Compiler::new();
    let ast1 = compiler.compile_string(source1, "comp1.rsx").expect("Should compile");
    let ast2 = compiler.compile_string(source2, "comp2.rsx").expect("Should compile");
    
    // Generate code for both
    let mut codegen = CodeGenerator::new();
    let code1 = codegen.generate_rust_code(&ast1).expect("Should generate");
    let code2 = codegen.generate_rust_code(&ast2).expect("Should generate");
    
    assert!(code1.contains("component1"));
    assert!(code2.contains("component2"));
}

#[test]
fn test_code_generation_types() {
    let source = r#"
struct Props {
    name: String,
}

fn Greeting(props: Props) -> Element {
    <h1>Hello, {props.name}</h1>
}
"#;
    
    let mut compiler = Compiler::new();
    let ast = compiler.compile_string(source, "test.rsx").expect("Should compile");
    
    let mut codegen = CodeGenerator::new();
    let code = codegen.generate_rust_code(&ast).expect("Should generate");
    
    // Verify struct and component are generated
    assert!(code.contains("pub struct Props"));
    assert!(code.contains("pub fn greeting"));
}
