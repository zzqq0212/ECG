use std::sync::atomic::AtomicBool;

pub static RUNNING: AtomicBool = AtomicBool::new(true);
pub static mut IS_DEBUG: bool = false;
// pub static DISABLE_CPU_BINDING_VAR: &str = "DISABLE_CPU_BINDING";
