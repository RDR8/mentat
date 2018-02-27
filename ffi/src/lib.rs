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
use std::os::raw::{
    c_char,
    c_void,
};

pub use mentat::{
    NamespacedKeyword,
    HasSchema,
    Store,
    TxObserver,
};

pub mod utils;

pub use utils::strings::{
    c_char_to_string,
    string_to_c_char,
};

#[repr(C)]
pub struct AttributeList {
    pub attributes: Box<[i64]>,
    pub len: usize
}

#[repr(C)]
pub struct ExternTxReport {
    pub txid: i64,
    pub changes: AttributeList,
}

#[repr(C)]
pub struct ExternTxReportList {
    pub reports: Box<[ExternTxReport]>,
    pub len: usize
}

#[repr(C)]
pub struct Callback {
    pub obj: *mut c_void,
    pub destroy: extern fn(obj: *mut c_void),
    pub callback_fn: extern fn(obj: *mut c_void, key: *const c_char, reports: *mut ExternTxReportList),
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
pub unsafe extern "C" fn store_register_observer(store: *mut Store, key: *const c_char, attributes: *const AttributeList, callback: *mut Callback) {
    let store = &mut*store;
    let callback = &mut*callback;
    let attrs = &*attributes;
    let mut attribute_set = BTreeSet::new();
    for attr in attrs.attributes.into_iter() {
        attribute_set.insert(*attr);
    }
    let key = c_char_to_string(key);
    let tx_observer = TxObserver::new(attribute_set, move |obs_key, batch| {
        println!("observer function called {:?}: {:?}", obs_key, batch);
        let extern_reports: Vec<ExternTxReport> = batch.iter().map(|report| {
            let changes: Vec<i64> = report.changeset.iter().map(|i|i.clone()).collect();
            let len = changes.len();
            let changelist = AttributeList {
                attributes: changes.into_boxed_slice(),
                len: len,
            };
            ExternTxReport {
                txid: report.tx_id.clone(),
                changes: changelist,
            }
        }).collect();
        let len = extern_reports.len();
        let reports = ExternTxReportList {
            reports: extern_reports.into_boxed_slice(),
            len: len,
        };

        (callback.callback_fn)(callback.obj, string_to_c_char(obs_key), Box::into_raw(Box::new(reports)));
    });
    store.register_observer(key, tx_observer);
}

#[no_mangle]
pub unsafe extern "C" fn store_unregister_observer(store: *mut Store, key: *const c_char) {
    let store = &mut*store;
    let key = c_char_to_string(key);
    store.unregister_observer(&key);
}
