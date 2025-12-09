use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use parking_lot::RwLock;

pub type SignalId = usize;

#[derive(Debug, Clone)]
pub struct Signal<T> {
    id: SignalId,
    value: Rc<RefCell<T>>,
    dependents: Rc<RwLock<Vec<SignalId>>>,
}

impl<T> Signal<T> {
    pub fn new(value: T) -> Self {
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        Self {
            id,
            value: Rc::new(RefCell::new(value)),
            dependents: Rc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        // Track dependency (simplified - would need current computation context)
        self.value.borrow().clone()
    }
    
    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = value;
        self.notify_dependents();
    }
    
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        f(&mut *self.value.borrow_mut());
        self.notify_dependents();
    }
    
    fn notify_dependents(&self) {
        let dependents = self.dependents.read();
        for &dependent_id in dependents.iter() {
            // Notify dependent (simplified - would need signal registry)
        }
    }
    
    pub fn id(&self) -> SignalId {
        self.id
    }
}

pub struct SignalRegistry {
    signals: HashMap<SignalId, Box<dyn std::any::Any>>,
    dependency_graph: HashMap<SignalId, Vec<SignalId>>,
}

impl SignalRegistry {
    pub fn new() -> Self {
        Self {
            signals: HashMap::new(),
            dependency_graph: HashMap::new(),
        }
    }
    
    pub fn register<T: 'static>(&mut self, signal: Signal<T>) {
        self.signals.insert(signal.id(), Box::new(signal));
    }
    
    pub fn add_dependency(&mut self, signal_id: SignalId, _dependent_id: SignalId) {
        self.dependency_graph
            .entry(signal_id)
            .or_insert_with(Vec::new);
    }
    
    pub fn get_dependents(&self, signal_id: SignalId) -> Option<&Vec<SignalId>> {
        self.dependency_graph.get(&signal_id)
    }
}

pub fn create_signal<T>(value: T) -> Signal<T> {
    Signal::new(value)
}

pub fn create_derived<F, T>(f: F) -> Signal<T>
where
    F: Fn() -> T + 'static,
    T: Clone + 'static,
{
    // Simplified - would track dependencies during computation
    Signal::new(f())
}

pub fn create_computed<F, T>(signal: &Signal<T>, f: F) -> Signal<T>
where
    F: Fn(&T) -> T + 'static,
    T: Clone + 'static,
{
    let value = signal.get();
    Signal::new(f(&value))
}
