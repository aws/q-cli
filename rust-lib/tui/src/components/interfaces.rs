pub trait PickerComponent {
    fn new<I, T>(options: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>;
    fn selected(&self) -> Option<usize>;
    fn set_selected(&mut self, index: usize);
    fn options(&self) -> &Vec<String>;
    fn selected_item(&self) -> Option<&String> {
        self.selected().map(|index| &self.options()[index])
    }
}
