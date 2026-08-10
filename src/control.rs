pub mod control_settings;
pub mod core;
pub mod decimating;
pub mod monolithic;
pub mod unchecked_double_mapped_queue;

pub trait Controller {
    fn run(&mut self);
}
