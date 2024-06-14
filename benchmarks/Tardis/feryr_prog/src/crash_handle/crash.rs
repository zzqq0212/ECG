#[derive(Default, Clone, Debug)]
pub struct Crash {
    _vm_id: u64,
    _log: String,
    _crash_type: String,
    _location: String,
    _hash: String,
}
impl Crash {
    pub fn new(err_msg: String) -> Crash {
        let crash_info = Crash {
            _vm_id: 0,
            _log: err_msg.clone(),
            _crash_type: String::from(""),
            _location: String::from(""),
            _hash: String::from("".to_owned()),
        };

        return crash_info;
    }
    pub fn replay() -> Result<bool, failure::Error> {
        Ok(true)
    }
}
