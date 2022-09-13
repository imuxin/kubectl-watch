use crate::diff::abs;

pub trait Process {
    fn process(self, l: &mut abs::DynamicObject, r: &mut abs::DynamicObject);
}

pub struct Pipeline {
    tasks: Vec<fn(&mut abs::DynamicObject, &mut abs::DynamicObject)>,
}

impl Pipeline {
    pub fn init() -> Self {
        let mut p = Pipeline { tasks: vec![] };
        p.add_task(exclude_types);
        p
    }
    pub fn add_task(&mut self, task: fn(&mut abs::DynamicObject, &mut abs::DynamicObject)) {
        self.tasks.push(task);
    }
}

impl Process for Pipeline {
    fn process(self, l: &mut abs::DynamicObject, r: &mut abs::DynamicObject) {
        for task in self.tasks {
            task(l, r);
        }
    }
}

pub fn exclude_managed_fields(l: &mut abs::DynamicObject, r: &mut abs::DynamicObject) {
    l.exclude_managed_fields();
    r.exclude_managed_fields();
}

pub fn exclude_types(l: &mut abs::DynamicObject, r: &mut abs::DynamicObject) {
    l.exclude_types();
    r.exclude_types();
}
