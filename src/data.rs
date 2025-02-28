use std::{
    any::{Any, TypeId},
    ops::Deref,
    sync::Arc,
};

pub struct Data<T> {
    pub data_type_id: TypeId,
    pub inner_data: Arc<T>,
}

impl<T: 'static> Data<T> {
    pub fn new(data: T) -> Self {
        let data_type_id = data.type_id();
        let inner_data = Arc::new(data);

        Self {
            data_type_id,
            inner_data,
        }
    }
}

impl<T> Clone for Data<T> {
    fn clone(&self) -> Self {
        Self {
            data_type_id: self.data_type_id.clone(),
            inner_data: self.inner_data.clone(),
        }
    }
}

impl<T> Deref for Data<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner_data
    }
}
