pub trait Formatable {
    fn formatted(&self, indent_level: usize) -> String;
}
