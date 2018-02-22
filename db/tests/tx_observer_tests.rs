// Copyright 2018 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.
extern crate mentat_db;

use std::cell::{
    RefCell
};
use std::collections::{
    BTreeSet,
};
use std::ops::Deref;
use std::rc::{
    Rc,
};

use mentat_db::{
    BatchedTransaction,
    TxObserver,
    TxObservationService,
};

fn get_registered_observer_attributes() -> BTreeSet<Entid> {
    let mut registered_attrs = BTreeSet::new();
    registered_attrs.insert(100);
    registered_attrs.insert(200);
    registered_attrs.insert(300);
    registered_attrs
}

#[test]
fn test_register_observer() {
    let mut observer_service = TxObservationService::default();
    let key = "Test Observing".to_string();
    let registered_attrs = BTreeSet::new();

    let tx_observer = TxObserver::new(move |_obs_key, _batch| {});

    observer_service.register(tx_observer, key.clone(), registered_attrs.clone());
    assert!(observer_service.is_registered(&key));
}

#[test]
fn test_deregister_observer() {
    let mut observer_service = TxObservationService::default();
    let key = "Test Observing".to_string();
    let registered_attrs = BTreeSet::new();

    let tx_observer = TxObserver::new(move |_obs_key, _batch| {});

    observer_service.register(tx_observer, key.clone(), registered_attrs.clone());
    assert!(observer_service.is_registered(&key));

    observer_service.deregister(&key);

    assert!(!observer_service.is_registered(&key));
}

#[test]
fn test_observer_notified_on_registered_change() {
    let mut observer_service = TxObservationService::default();
    let key = "Test Observing".to_string();
    let register_attrs = get_registered_observer_attributes();

    let txids = Rc::new(RefCell::new(Vec::new()));
    let changes = Rc::new(RefCell::new(Vec::new()));
    let called_key: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    let mut_txids = Rc::clone(&txids);
    let mut_changes = Rc::clone(&changes);
    let mut_key = Rc::clone(&called_key);
    let tx_observer = TxObserver::new(move |obs_key, batch| {
        let mut k = mut_key.borrow_mut();
        *k = Some(obs_key.clone());
        let mut t = mut_txids.borrow_mut();
        let mut c = mut_changes.borrow_mut();
        for (tx, changes) in batch.get().iter() {
            t.push(tx.clone());
            c.push(changes.clone());
        }
        t.sort();
    });

    observer_service.register(tx_observer, key.clone(), registered_attrs.clone());
    assert!(observer_service.is_registered(&key));

    let mut tx_set_1 = BTreeSet::new();
    tx_set_1.insert(100);
    tx_set_1.insert(400);
    tx_set_1.insert(700);
    let mut tx_set_2 = BTreeSet::new();
    tx_set_2.insert(200);
    tx_set_2.insert(300);
    let mut tx_set_3 = BTreeSet::new();
    tx_set_3.insert(600);
    let mut batch = BatchedTransaction::default();
    batch.add_transact(10, tx_set_1);
    batch.add_transact(11, tx_set_2);
    batch.add_transact(12, tx_set_3);
    observer_service.transaction_did_commit(&Some(batch));

    let val = called_key.deref();
    assert_eq!(val, &RefCell::new(Some(key.clone())));
    let t = txids.deref();
    assert_eq!(t, &RefCell::new(vec![10, 11]));

    let mut change_set_1 = BTreeSet::new();
    change_set_1.insert(100);
    let mut change_set_2 = BTreeSet::new();
    change_set_2.insert(200);
    change_set_2.insert(300);
    let c = changes.deref();
    assert_eq!(c, &RefCell::new(vec![change_set_1, change_set_2]));
}

#[test]
fn test_observer_not_notified_on_unregistered_change() {
    let mut observer_service = TxObservationService::default();
    let key = "Test Observing".to_string();
    let register_attrs = get_registered_observer_attributes();

    let txids = Rc::new(RefCell::new(Vec::new()));
    let changes = Rc::new(RefCell::new(Vec::new()));
    let called_key: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    let mut_txids = Rc::clone(&txids);
    let mut_changes = Rc::clone(&changes);
    let mut_key = Rc::clone(&called_key);
    let tx_observer = TxObserver::new(move |obs_key, batch| {
        let mut k = mut_key.borrow_mut();
        *k = Some(obs_key.clone());
        let mut t = mut_txids.borrow_mut();
        let mut c = mut_changes.borrow_mut();
        for (tx, changes) in batch.get().iter() {
            t.push(tx.clone());
            c.push(changes.clone());
        }
        t.sort();
    });

    observer_service.register(tx_observer, key.clone(), registered_attrs.clone());
    assert!(observer_service.is_registered(&key));

    let mut tx_set_1 = BTreeSet::new();
    tx_set_1.insert(101);
    tx_set_1.insert(401);
    tx_set_1.insert(701);
    let mut tx_set_2 = BTreeSet::new();
    tx_set_2.insert(201);
    tx_set_2.insert(301);
    let mut tx_set_3 = BTreeSet::new();
    tx_set_3.insert(601);
    let mut batch = BatchedTransaction::default();
    batch.add_transact(10, tx_set_1);
    batch.add_transact(11, tx_set_2);
    batch.add_transact(12, tx_set_3);
    observer_service.transaction_did_commit(&Some(batch));

    let val = called_key.deref();
    assert_eq!(val, &RefCell::new(None));
    let t = txids.deref();
    assert_eq!(t, &RefCell::new(vec![]));
    let c = changes.deref();
    assert_eq!(c, &RefCell::new(vec![]));
}
