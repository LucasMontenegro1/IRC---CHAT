use std::cell::RefCell;

use glib::once_cell::sync::Lazy;
use gtk::glib;
use gtk::glib::{ParamSpec, ParamSpecString, Value};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk4 as gtk;

#[derive(Default)]
pub struct StringObject {
    string: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for StringObject {
    const NAME: &'static str = "MyGtkAppStringObject";
    type Type = super::StringObject;
}

impl StringObject {
    pub fn get_string(&self) -> String {
        self.string.take()
    }
}

impl ObjectImpl for StringObject {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> =
            Lazy::new(|| vec![ParamSpecString::builder("string").build()]);
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "string" => {
                let input_string = value
                    .get()
                    .expect("The value needs to be of type `String`.");
                self.string.replace(input_string);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "string" => self.string.borrow_mut().to_value(),
            _ => unimplemented!(),
        }
    }
}
