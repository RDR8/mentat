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
use std::ops::Deref;
use std::os::raw::{
    c_char,
    c_void,
};
use std::slice;
use std::thread;
use std::time;

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
#[derive(Debug)]
pub struct AttributeList {
    pub attributes: Box<[i64]>,
    pub len: usize
}

#[repr(C)]
#[derive(Debug)]
pub struct ExternTxReport {
    pub txid: i64,
    pub changes: AttributeList,
}

#[repr(C)]
#[derive(Debug)]
pub struct ExternTxReportList {
    pub reports: Box<[ExternTxReport]>,
    pub len: usize
}

#[derive(Debug)]
struct CallbackWrapper(Callback);
impl Deref for CallbackWrapper {
    type Target = Callback;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for CallbackWrapper {
    fn drop(&mut self) {
        (self.destroy)(self.obj);
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Callback {
    pub obj: *mut c_void,
    pub destroy: extern fn(obj: *mut c_void),
    pub callback_fn: extern fn(obj: *mut c_void, key: *const c_char, reports: *mut ExternTxReportList),
}

unsafe impl Send for Callback {}

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
pub unsafe extern "C" fn store_register_observer(store: *mut Store, key: *const c_char, attributes: *const i64, attributes_len: usize, callback: Callback) {
    println!("Registering observer");
    let store = &mut*store;
    println!("got store");
    let callback = CallbackWrapper(callback);
    println!("got callback");
    let mut attribute_set = BTreeSet::new();
    let slice = slice::from_raw_parts(attributes, attributes_len);
    println!("slice {:?}", slice);
    for i in 0..attributes_len {
        let attr = slice[i];
        println!("adding {} to attribute set", attr);
        attribute_set.insert(attr);
    }
    println!("Attribute set  {:?}", attribute_set);
    let key = c_char_to_string(key);
    println!("Registering observer for {:?}", key);
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
        println!("ExternReports {:?}", extern_reports);
        let len = extern_reports.len();
        let reports = ExternTxReportList {
            reports: extern_reports.into_boxed_slice(),
            len: len,
        };
        println!("reports {:?}", reports);
        println!("calling callback {:?}", callback.obj);
        (callback.callback_fn)(callback.obj, string_to_c_char(obs_key), Box::into_raw(Box::new(reports)));
        println!("callback called");
    });
    println!("TxObserver created");
    store.register_observer(key, tx_observer);
}

#[no_mangle]
pub unsafe extern "C" fn store_unregister_observer(store: *mut Store, key: *const c_char) {
    println!("Unregistering observer");
    let store = &mut*store;
    let key = c_char_to_string(key);
    println!("Unregistering observer for {:?}", key);
    store.unregister_observer(&key);
    println!("{:?} unregistered", key);
}

#[no_mangle]
pub unsafe extern "C" fn store_entid_for_attribute(store: *mut Store, attr: *const c_char) -> i64 {
    println!("store_entid_for_attribute");
    let store = &mut*store;
    let attr_name = c_char_to_string(attr);
    println!("attr_name {:?}", attr_name);
    let parts: Vec<&str> = attr_name.split("/").collect();
    let kw = NamespacedKeyword::new(parts[0], parts[1]);
    println!("kw {:?}", kw);
    let entid = store.conn().current_schema().get_entid(&kw).unwrap();
    println!("entid {:?}", entid);
    entid.into()
}
