mod collapsible_picker;
mod filterable_picker;
mod generic_picker;

pub use collapsible_picker::CollapsiblePicker;
pub use filterable_picker::FilterablePicker;
pub use generic_picker::Picker;

pub trait PickerComponent {
    fn new<I, T>(options: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>;
    fn selected(&self) -> Option<usize>;
    fn options(&self) -> &Vec<String>;
}
