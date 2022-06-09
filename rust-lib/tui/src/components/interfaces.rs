pub trait PickerComponent {
    fn new<I, T>(options: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>;
    fn selected(&self) -> Option<usize>;
    fn options(&self) -> &Vec<String>;
}
