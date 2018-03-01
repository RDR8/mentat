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
    c_int,
};
use std::slice;

pub use mentat::{
    NamespacedKeyword,
    HasSchema,
    Store,
    TxObserver,
};

pub mod utils;
pub mod android;

pub use utils::strings::{
    c_char_to_string,
    string_to_c_char,
};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AttributeList {
    pub attributes: Box<[i64]>,
    pub len: usize
}

#[repr(C)]
#[derive(Debug, Clone)]
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
pub unsafe extern "C" fn store_register_observer(store: *mut Store,
                                                   key: *const c_char,
                                                   attributes: *const i64,
                                                   attributes_len: usize,
                                                   callback: extern fn(key: *const c_char, reports: *mut ExternTxReportList)) {
    let store = &mut*store;
    let mut attribute_set = BTreeSet::new();
    let slice = slice::from_raw_parts(attributes, attributes_len);
    for i in 0..attributes_len {
        let attr = slice[i];
        attribute_set.insert(attr);
    }
    let key = c_char_to_string(key);
    let tx_observer = TxObserver::new(attribute_set, move |obs_key, batch| {
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
        callback(string_to_c_char(obs_key), Box::into_raw(Box::new(reports)));
    });
    store.register_observer(key, tx_observer);
}

#[no_mangle]
pub unsafe extern "C" fn store_unregister_observer(store: *mut Store, key: *const c_char) {
    let store = &mut*store;
    let key = c_char_to_string(key);
    store.unregister_observer(&key);
}

#[no_mangle]
pub unsafe extern "C" fn store_entid_for_attribute(store: *mut Store, attr: *const c_char) -> i64 {
    let store = &mut*store;
    let attr_name = c_char_to_string(attr);
    let parts: Vec<&str> = attr_name.split("/").collect();
    let kw = NamespacedKeyword::new(parts[0], parts[1]);
    let entid = store.conn().current_schema().get_entid(&kw).unwrap();
    entid.into()
}

#[no_mangle]
pub unsafe extern "C" fn tx_report_list_entry_at(tx_report_list: *mut ExternTxReportList, index: c_int) -> *const ExternTxReport {
    let tx_report_list = &*tx_report_list;
    let index = index as usize;
    let report = Box::new(tx_report_list.reports[index].clone());
    Box::into_raw(report)
}

#[no_mangle]
pub unsafe extern "C" fn changelist_entry_at(attribute_list: *mut AttributeList, index: c_int) -> i64 {
    let attribute_list = &*attribute_list;
    let index = index as usize;
    attribute_list.attributes[index].clone()
}
