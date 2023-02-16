use std::collections::HashMap;

#[allow(type_alias_bounds)]
pub type Memory<T: UID> = HashMap<String, Vec<T>>;

pub trait UID {
    fn uid(&self) -> String;
    fn resource_version(&self) -> String;
}

pub trait Database<T: UID> {
    fn do_insert(&mut self, obj: T);
    fn sibling(&self, obj: &T) -> Option<&T>;
    fn index_of(&self, obj: &T) -> usize;
}

impl<T: UID> Database<T> for Memory<T> {
    fn do_insert(&mut self, obj: T) {
        let empty_list = Vec::<T>::new();
        self.entry(obj.uid()).or_insert(empty_list);
        if let Some(list) = self.get_mut(&obj.uid().clone()) {
            list.push(obj);
        }
    }

    fn index_of(&self, obj: &T) -> usize {
        for (i, item) in self.get(&obj.uid().clone()).unwrap().iter().enumerate() {
            if item.resource_version() == obj.resource_version() {
                return i;
            }
        }
        0
    }

    fn sibling(&self, obj: &T) -> Option<&T> {
        let pos = self.index_of(obj);
        if pos == 0 {
            return None;
        }
        self.get(&obj.uid().clone()).unwrap().get(pos - 1)
    }
}
