mod imp;

use gtk4::glib;
use gtk4::glib::{Object, ObjectExt};

glib::wrapper! {
    pub struct StringObject(ObjectSubclass<imp::StringObject>);
}

impl StringObject {
    pub fn new(string: String) -> Self {
        Object::new(&[("string", &string)])
    }

    pub fn get_string(self) -> String {
        self.property::<String>("string")
        // self.set_property("string",old_number);
    }

    pub fn do_some(self) {
        let old_number = self.property::<String>("string");
        println!("{}", old_number);
        // self.set_property("string",old_number);
    }
}
