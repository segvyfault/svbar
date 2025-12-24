pub trait ModuleInfo {
    fn display(&mut self) -> String;
    fn clean_up(&mut self) {}
}
