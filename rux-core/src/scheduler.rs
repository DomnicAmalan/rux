use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Immediate = 0,
    UserBlocking = 1,
    Normal = 2,
    Low = 3,
    Idle = 4,
}

pub struct Fiber {
    pub id: FiberId,
    pub priority: Priority,
    pub work: Box<dyn FnOnce() + Send>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FiberId(pub usize);

pub struct Scheduler {
    work_queue: VecDeque<Fiber>,
    current_fiber: Option<Fiber>,
    deadline: Option<Instant>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            work_queue: VecDeque::new(),
            current_fiber: None,
            deadline: None,
        }
    }
    
    pub fn schedule(&mut self, fiber: Fiber) {
        // Insert in priority order
        let priority = fiber.priority as usize;
        let mut insert_pos = 0;
        for (i, queued) in self.work_queue.iter().enumerate() {
            if queued.priority as usize > priority {
                insert_pos = i;
                break;
            }
            insert_pos = i + 1;
        }
        self.work_queue.insert(insert_pos, fiber);
    }
    
    pub fn work_loop(&mut self, deadline: Instant) {
        self.deadline = Some(deadline);
        
        while let Some(mut fiber) = self.get_next_unit_of_work() {
            if !self.has_time_remaining() {
                // Reschedule for later
                self.schedule(fiber);
                break;
            }
            
            // Execute work
            (fiber.work)();
        }
    }
    
    fn get_next_unit_of_work(&mut self) -> Option<Fiber> {
        self.work_queue.pop_front()
    }
    
    fn has_time_remaining(&self) -> bool {
        if let Some(deadline) = self.deadline {
            Instant::now() < deadline
        } else {
            true
        }
    }
    
    pub fn should_yield(&self) -> bool {
        !self.has_time_remaining()
    }
    
    pub fn flush_work(&mut self) {
        while let Some(fiber) = self.work_queue.pop_front() {
            (fiber.work)();
        }
    }
}

pub struct TimeSlice {
    pub duration: Duration,
}

impl TimeSlice {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
    
    pub fn has_time_remaining(&self, start: Instant) -> bool {
        Instant::now() - start < self.duration
    }
}

pub fn schedule_work(priority: Priority, work: impl FnOnce() + Send + 'static) {
    // Global scheduler (simplified)
    let _fiber = Fiber {
        id: FiberId(0), // Would generate unique ID
        priority,
        work: Box::new(work),
    };
    // Would add to global scheduler
}

pub fn should_yield() -> bool {
    // Check if should yield to browser/main thread
    false // Simplified
}
