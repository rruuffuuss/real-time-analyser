pub mod control_settings;
pub mod core;
pub mod decimating;
pub mod monolithic;

pub trait Controller {
    fn run(&mut self);
}
