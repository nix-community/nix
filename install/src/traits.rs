pub trait Step: std::fmt::Debug {
    fn apply(&self) -> Result<(), ()>;
    fn dry_apply(&self) -> Result<(), ()>;
    fn delete(&self) -> Result<(), ()>;
}
