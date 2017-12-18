#![allow(dead_code)]

use std::borrow::{Borrow, ToOwned};
use std::boxed::Box;
use std::collections::HashMap;
use std::hash::Hash;

use error;
use error::FileLocation;
use error::Error;
use error::ErrorKind::*;
use types::*;


pub struct IDTracker<T>
where T: Hash + Eq + Clone {
    next_id: usize,
    name_to_ident: HashMap<T, usize>,
    ident_to_name: HashMap<usize, T>,
}

impl<T> IDTracker<T>
where T: Hash + Eq + Clone {
    pub fn new() -> IDTracker<T> {
        IDTracker {
            next_id: 0usize,
            name_to_ident: HashMap::new(),
            ident_to_name: HashMap::new(),
        }
    }

    fn new_identifier(&mut self, name: T) -> usize {
        let id = self.next_id.clone();
        self.name_to_ident.insert(name.clone(), id);
        self.ident_to_name.insert(id, name);
        self.next_id += 1;
        id
    }

    pub fn get_id_nomake<Q: ?Sized>(&self, name: &Q) -> Option<&usize>
    where Q: Borrow<T> {
        self.name_to_ident.get(name.borrow())
    }

    pub fn get_id<Q: ?Sized>(&mut self, name: &Q) -> usize
    where Q: Borrow<T> {
        let name_copy = name.to_owned();
        if let Some(id) = self.name_to_ident.get(name_copy.borrow()) {
            return id.clone();
        }
        self.new_identifier(name_copy.borrow().clone())
    }

    pub fn get_name(&self, id: &usize) -> Option<&T> {
        self.ident_to_name.get(id).clone()
    }

    pub fn remove(&mut self, id: &usize) {
        let name = self.ident_to_name.remove(id).unwrap();
        self.name_to_ident.remove(&name);
        self.next_id -= 1;
    }
}


pub enum ChainMap<K, V> {
    Child(HashMap<K, V>, Box<ChainMap<K, V>>),
    Base(HashMap<K, V>),
}

impl<K, V> ChainMap<K, V>
where K: Eq + Hash {
    pub fn new() -> ChainMap<K, V> {
        ChainMap::Base(HashMap::new())
    }

    pub fn new_child(self) -> ChainMap<K, V> {
        ChainMap::Child(HashMap::new(), Box::new(self))
    }

    pub fn parent(self) -> Result<ChainMap<K, V>, &'static str> {
        match self {
            ChainMap::Child(_, box parent) => Ok(parent),
            ChainMap::Base(_)              => Err("cannot take parent of base map"),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            &ChainMap::Child(ref map, box ref parent) => map.is_empty() && parent.is_empty(),
            &ChainMap::Base(ref map)                  => map.is_empty(),
        }
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where K: Borrow<Q>,
          Q: Hash + Eq {
        match self {
            &ChainMap::Child(ref map, box ref parent) => match map.get(key) {
                Some(val) => Some(val),
                None      => parent.get(key),
            },
            &ChainMap::Base(ref map)                  => map.get(key),
        }
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        match self {
            &mut ChainMap::Child(ref mut map, _) => map.insert(key, val),
            &mut ChainMap::Base(ref mut map)     => map.insert(key, val),
        }
    }
}


pub struct Bindings {
    next_local: usize,
    id_table: IDTracker<String>,
    val_table: ChainMap<usize, (MetaType, Option<MetaValue>)>,
    // base_types: ChainMap<usize, InternalType>,
    // term_types: ChainMap<usize, InternalType>,
    // form_types: ChainMap<usize, MetaType>,
    proven_wffs: ChainMap<usize, FormulaSchema>,
}

impl Bindings {
    pub fn new() -> Bindings {
        Bindings {
            next_local: 0usize,
            id_table: IDTracker::new(),
            val_table: ChainMap::new(),
            // base_types: ChainMap::new(),
            // term_types: ChainMap::new(),
            // form_types: ChainMap::new(),
            proven_wffs: ChainMap::new(),
        }
    }

    pub fn new_local(&mut self) -> usize {
        let res = self.next_local;
        self.next_local += 1;
        res
    }

    pub fn new_child(self) -> Bindings {
        Bindings {
            next_local: self.next_local,
            id_table: self.id_table,
            val_table: self.val_table.new_child(),
            // base_types: self.base_types.new_child(),
            // term_types: self.term_types.new_child(),
            // form_types: self.form_types.new_child(),
            proven_wffs: self.proven_wffs.new_child(),
        }
    }

    pub fn parent(self) -> Bindings {
        Bindings {
            next_local: self.next_local,
            id_table: self.id_table,
            val_table: self.val_table.parent().unwrap(),
            // base_types: self.base_types.parent().unwrap(),
            // term_types: self.term_types.parent().unwrap(),
            // form_types: self.form_types.parent().unwrap(),
            proven_wffs: self.proven_wffs.parent().unwrap(),
        }
    }

    pub fn get_id(&mut self, name: &str) -> usize {
        self.id_table.get_id(&name.to_owned())
    }

    pub fn get_name(&self, id: &usize) -> Option<&String> {
        self.id_table.get_name(id)
    }

    pub fn get_type(&self, id: &usize) -> Option<MetaType> {
        if let Some(&(ref mtype, _)) = self.val_table.get(id) { Some(mtype.clone()) } else { None }
    }

    pub fn get_value(&self, id: &usize) -> Option<MetaValue> {
        if let Some(&(_, ref itype_q)) = self.val_table.get(id) { itype_q.clone() } else { None }
    }

    pub fn get_type_value(&self, id: &usize) -> Option<(MetaType, Option<MetaValue>)> {
        if let Some(&ref pair) = self.val_table.get(id) { Some(pair.clone()) } else { None }
    }

    pub fn get_theorem(&self, id: &usize) -> Option<&FormulaSchema> {
        self.proven_wffs.get(id)
    }

    pub fn insert_object_noval(&mut self, id: usize, mtype: MetaType, context: &FileLocation) -> error::Result<usize> {
        if let None = self.val_table.insert(id, (mtype, None)) {
            Ok(id)
        } else {
            Err(Error::new(BindingExists { name: self.get_name(&id).unwrap().clone() }, context))
        }
    }

    pub fn insert_object(&mut self, id: usize, mtype: MetaType, mval: MetaValue, context: &FileLocation) -> error::Result<usize> {
        if let None = self.val_table.insert(id, (mtype, Some(mval))) {
            Ok(id)
        } else {
            Err(Error::new(BindingExists { name: self.get_name(&id).unwrap().clone() }, context))
        }
    }

    pub fn insert_object_anytype(&mut self, id: usize, mtype: MetaType, mval: MetaValue, context: &FileLocation) -> error::Result<usize> {
        if let Some((_, None)) = self.val_table.insert(id, (mtype, Some(mval))) {
            Ok(id)
        } else {
            Err(Error::new(BindingExists { name: self.get_name(&id).unwrap().clone() }, context))
        }
    }

    pub fn insert_theorem(&mut self, id: usize, stmt: FormulaSchema, context: &FileLocation) -> error::Result<usize> {
        if let Some(_) = self.proven_wffs.insert(id, stmt) {
            Err(Error::new(BindingExists { name: self.get_name(&id).unwrap().clone() }, context))
        } else {
            Ok(id)
        }
    }
}


pub trait RLangRepr {
    fn repr(&self, globals: &Bindings) -> String;
}

impl RLangRepr for InternalType {
    fn repr(&self, globals: &Bindings) -> String {
        match self {
            &InternalType::Named(name)                              => {
                match name {
                    Ident::Global(id) => globals.get_name(&id).unwrap().clone(),
                    Ident::Local(local_id) => format!("#{}", local_id),
                }
            },
            &InternalType::Func(box ref arg_type, box ref ret_type) => format!("({} -> {})", arg_type.repr(globals), ret_type.repr(globals)),
        }
    }
}

impl RLangRepr for MetaType {
    fn repr(&self, globals: &Bindings) -> String {
        match self {
            &MetaType::Type                                    => String::from("Type"),
            &MetaType::Term(ref itype)                         => {
                format!("(Term {})", itype.repr(globals))
            },
            &MetaType::Formula(ref arg_types)                  => {
                format!("(Formula {})", arg_types.iter().rev().fold(String::new(), |acc, ref itype| {
                    format!("{} {}", acc, itype.repr(globals))
                }))
            },
            &MetaType::Schema(ref arg_types, box ref ret_type) => {
                format!("(Schema {} to {})", arg_types.iter().rev().fold(String::new(), |acc, ref mtype| {
                    format!("{} {}", acc, mtype.repr(globals))
                }), ret_type.repr(globals))
            },
        }
    }
}