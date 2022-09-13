use crate::diff::dynamic_object;

pub trait Process {
    fn process(self, l: &mut dynamic_object::DynamicObject, r: &mut dynamic_object::DynamicObject);
}

pub struct Pipeline {
    tasks: Vec<fn(&mut dynamic_object::DynamicObject, &mut dynamic_object::DynamicObject)>,
}

impl Pipeline {
    pub fn init() -> Self {
        let mut p = Pipeline { tasks: vec![] };
        p.add_task(exclude_types);
        p
    }
    pub fn add_task(&mut self, task: fn(&mut dynamic_object::DynamicObject, &mut dynamic_object::DynamicObject)) {
        self.tasks.push(task);
    }
}

impl Process for Pipeline {
    fn process(self, l: &mut dynamic_object::DynamicObject, r: &mut dynamic_object::DynamicObject) {
        for task in self.tasks {
            task(l, r);
        }
    }
}

pub fn exclude_managed_fields(l: &mut dynamic_object::DynamicObject, r: &mut dynamic_object::DynamicObject) {
    l.exclude_managed_fields();
    r.exclude_managed_fields();
}

pub fn exclude_types(l: &mut dynamic_object::DynamicObject, r: &mut dynamic_object::DynamicObject) {
    l.exclude_types();
    r.exclude_types();
}
