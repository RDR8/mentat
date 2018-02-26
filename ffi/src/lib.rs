// Copyright 2018 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

extern crate mentat;

use std::collections::{
    BTreeSet,
};
use std::os::raw::c_char;

pub use mentat::{
    NamespacedKeyword,
    HasSchema,
    Store,
    TxObserver,
};

pub mod utils;

pub use utils::strings::{
    c_char_to_string,
};

pub struct AttributeList {
    pub attributes: Box<[*const c_char]>,
    pub len: usize
}

#[no_mangle]
pub extern "C" fn new_store(uri: *const c_char) -> *mut Store {
    let uri = c_char_to_string(uri);
    let store = Store::open(&uri).expect("expected a store");
    Box::into_raw(Box::new(store))
}

#[no_mangle]
pub unsafe extern "C" fn store_destroy(store: *mut Store) {
    let _ = Box::from_raw(store);
}

#[no_mangle]
pub unsafe extern "C" fn store_register_observer(store: *mut Store, key: *const c_char, attributes: *const AttributeList) {
    let store = &mut*store;
    let attrs = &*attributes;
    let mut attribute_set = BTreeSet::new();
    for attr in attrs.attributes.into_iter() {
        let kw_str = c_char_to_string(*attr);
        let parts: Vec<&str> = kw_str.split('/').collect();
        assert!(parts.len() == 2);
        let kw = NamespacedKeyword::new(parts[0], parts[1]);
        if let Some(kw_entid) = store.conn().current_schema().get_entid(&kw) {
            attribute_set.insert(kw_entid.into());
        }
    }
    let key = c_char_to_string(key);
    let tx_observer = TxObserver::new(attribute_set, move |obs_key, batch| {
        println!("observer function called {:?}: {:?}", obs_key, batch);
    });
    store.register_observer(key, tx_observer);
}

#[no_mangle]
pub unsafe extern "C" fn store_unregister_observer(store: *mut Store, key: *const c_char) {
    let store = &mut*store;
    let key = c_char_to_string(key);
    store.unregister_observer(&key);
}
