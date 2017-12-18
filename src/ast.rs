#![allow(dead_code)]

// use std::boxed::Box;
use std::collections::HashMap;

use error;
use error::FileLocation;
use error::Error;
use error::ErrorKind::*;
pub use state::{Bindings, IDTracker, RLangRepr};
pub use types::{InternalType, MetaType, MetaValue, Term, Formula, FormulaSchema};
pub use types::Ident::*;


pub struct LocalBindings {
    glob_to_loc: HashMap<usize, usize>,
    loc_to_glob: HashMap<usize, usize>,
    local_types: HashMap<usize, MetaType>,
}

impl LocalBindings {
    pub fn new() -> LocalBindings {
        LocalBindings {
            glob_to_loc: HashMap::new(),
            loc_to_glob: HashMap::new(),
            local_types: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.glob_to_loc.is_empty()
    }

    pub fn get_local(&self, id: &usize) -> Option<usize> {
        if let Some(&local_id) = self.glob_to_loc.get(id) { Some(local_id) } else { None }
    }

    pub fn get_global(&self, local_id: &usize) -> Option<usize> {
        if let Some(&id) = self.loc_to_glob.get(local_id) { Some(id) } else { None }
    }

    pub fn get_type(&self, local_id: &usize) -> &MetaType {
        self.local_types.get(local_id).unwrap()
    }

    pub fn insert(&mut self, id: usize, local_id: usize, mtype: MetaType) -> usize {
        assert!(!self.glob_to_loc.contains_key(&id));
        self.glob_to_loc.insert(id, local_id);
        self.loc_to_glob.insert(local_id, id);
        self.local_types.insert(local_id, mtype);
        local_id
    }

    pub fn remove(&mut self, local_id: &usize) -> MetaType {
        let id = self.loc_to_glob.remove(local_id).unwrap();
        self.glob_to_loc.remove(&id).unwrap();
        self.local_types.remove(local_id).unwrap()
    }
}


pub struct TermBuilder {
    itype: InternalType,
    value: Term,
    location: FileLocation,
}

impl TermBuilder {
    pub fn symbol(id: usize, locals: &LocalBindings, globals: &Bindings, location: FileLocation) -> error::Result<TermBuilder> {
        if let Some(local_id) = locals.get_local(&id) {
            let mtype = locals.get_type(&local_id);
            if let &MetaType::Term(ref itype) = mtype {
                Ok(TermBuilder {
                    itype: itype.clone(),
                    value: Term::Symbol(Local(local_id)),
                    location: location,
                })
            } else {
                Err(Error::new(MTypeMismatch {
                    found: mtype.repr(globals),
                    expected: String::from("Term _"),
                }, &location))
            }
        } else if let Some(MetaType::Term(ref itype)) = globals.get_type(&id) {
            Ok(TermBuilder {
                itype: itype.clone(),
                value: Term::Symbol(Global(id)),
                location: location,
            })
        } else {
            Err(Error::new(NoBinding {
                name: globals.get_name(&id).unwrap().clone(),
            }, &location))
        }
    }

    pub fn application(function: TermBuilder, argument: TermBuilder, globals: &Bindings) -> error::Result<TermBuilder> {
        if let InternalType::Func(box arg_type, box ret_type) = function.itype {
            if argument.itype == arg_type {
                Ok(TermBuilder {
                    itype: ret_type,
                    value: Term::Application(Box::new(function.value), Box::new(argument.value)),
                    location: function.location,
                })
            } else {
                Err(Error::new(ITypeMismatch {
                    found: arg_type.repr(globals),
                    expected: argument.itype.repr(globals),
                }, &argument.location))
            }
        } else {
            Err(Error::new(ITypeMismatch {
                found: function.itype.repr(globals),
                expected: format!("{} -> _", argument.itype.repr(globals)),
            }, &function.location))
        }
    }
}


pub struct FormulaBuilder {
    arg_types: Vec<InternalType>,
    value: Formula,
    location: FileLocation,
}

impl FormulaBuilder {
    pub fn false_(location: FileLocation) -> error::Result<FormulaBuilder> {
        Ok(FormulaBuilder {
            arg_types: Vec::new(),
            value: Formula::False,
            location: location,
        })
    }

    pub fn relation(id: usize, locals: &LocalBindings, globals: &Bindings, location: FileLocation) -> error::Result<FormulaBuilder> {
        if let Some(local_id) = locals.get_local(&id) {
            let mtype = locals.get_type(&local_id);
            if let &MetaType::Formula(ref arg_types) = mtype {
                Ok(FormulaBuilder {
                    arg_types: arg_types.clone(),
                    value: Formula::Relation(Local(local_id)),
                    location: location,
                })
            } else {
                Err(Error::new(MTypeMismatch {
                    found: mtype.repr(globals),
                    expected: String::from("Formula _*"),
                }, &location))
            }
        } else if let Some(MetaType::Formula(ref arg_types)) = globals.get_type(&id) {
            Ok(FormulaBuilder {
                arg_types: arg_types.clone(),
                value: Formula::Relation(Global(id)),
                location: location,
            })
        } else {
            Err(Error::new(NoBinding {
                name: globals.get_name(&id).unwrap().clone(),
            }, &location))
        }
    }

    pub fn application(mut predicate: FormulaBuilder, term: TermBuilder, globals: &Bindings) -> error::Result<FormulaBuilder> {
        if let Some(term_type) = predicate.arg_types.pop() {
            if term.itype == term_type {
                Ok(FormulaBuilder {
                    arg_types: predicate.arg_types,
                    value: Formula::Application(Box::new(predicate.value), term.value),
                    location: predicate.location,
                })
            } else {
                Err(Error::new(ITypeMismatch {
                    found: term.itype.repr(globals),
                    expected: term_type.repr(globals),
                }, &term.location))
            }
        } else {
            Err(Error::new(MTypeMismatch {
                found: MetaType::Formula(Vec::new()).repr(globals),
                expected: format!("Formula {} _", term.itype.repr(globals)),
            }, &predicate.location))
        }
    }

    pub fn implication(lhs: FormulaBuilder, rhs: FormulaBuilder) -> error::Result<FormulaBuilder> {
        if lhs.arg_types.len() == 0 && rhs.arg_types.len() == 0 {
            Ok(FormulaBuilder {
                arg_types: lhs.arg_types,
                value: Formula::Implication(Box::new(lhs.value), Box::new(rhs.value)),
                location: lhs.location,
            })
        } else {
            Err(Error::new(UnboundImplication, &lhs.location))
        }
    }

    pub fn quantifier_prep(id: usize, itype: InternalType, locals: &mut LocalBindings, globals: &mut Bindings, location: FileLocation) -> error::Result<()> {
        if let None = locals.get_local(&id) {
            locals.insert(id, globals.new_local(), MetaType::Term(itype));
            Ok(())
        } else {
            Err(Error::new(BindingExists {
                name: globals.get_name(&id).unwrap().clone(),
            }, &location))
        }
    }

    pub fn universal_q(id: usize, itype: InternalType, formula: FormulaBuilder, locals: &mut LocalBindings, location: FileLocation) -> error::Result<FormulaBuilder> {
        let local_id = locals.get_local(&id).unwrap();
        locals.remove(&local_id);
        Ok(FormulaBuilder {
            arg_types: formula.arg_types,
            value: Formula::UniversalQ(local_id, itype, Box::new(formula.value)),
            location: location,
        })
    }

    pub fn value(self) -> Formula {
        self.value
    }
}


pub struct FSchemaBuilder {
    marg_types: Vec<MetaType>,
    iarg_types: Vec<InternalType>,
    value: FormulaSchema,
    location: FileLocation,
}

impl FSchemaBuilder {
    pub fn formula(formula: FormulaBuilder) -> error::Result<FSchemaBuilder> {
        Ok(FSchemaBuilder {
            marg_types: Vec::new(),
            iarg_types: formula.arg_types,
            value: FormulaSchema::Formula(formula.value),
            location: formula.location,
        })
    }

    pub fn schema_prep(id: usize, mtype: MetaType, locals: &mut LocalBindings, globals: &mut Bindings, location: FileLocation) -> error::Result<()> {
        if let None = locals.get_local(&id) {
            locals.insert(id, globals.new_local(), mtype);
            Ok(())
        } else {
            Err(Error::new(BindingExists {
                name: globals.get_name(&id).unwrap().clone(),
            }, &location))
        }
    }

    pub fn schema(id: usize, mtype: MetaType, mut schema: FSchemaBuilder, locals: &mut LocalBindings, location: FileLocation) -> error::Result<FSchemaBuilder> {
        let local_id = locals.get_local(&id).unwrap();
        locals.remove(&local_id);
        schema.marg_types.push(mtype.clone());
        Ok(FSchemaBuilder {
            marg_types: schema.marg_types,
            iarg_types: schema.iarg_types,
            value: FormulaSchema::Schema(local_id, mtype, Box::new(schema.value)),
            location: location,
        })
    }

    pub fn is_wff_schema(&self) -> bool {
        self.iarg_types.len() == 0
    }

    pub fn value(self) -> FormulaSchema {
        self.value
    }
}


// fn singleton<T>(x: T) -> Vec<T> {
//     let mut lst = Vec::new();
//     lst.push_front(x);
//     lst
// }


// fn concat_bindings<T>(mut lhs: Vec<T>, rhs: Vec<T>) -> Vec<T> {
//     'rloop: for id, val in rhs.iter() {
//         for id_, _ in lhs.iter() {
//             if id == id_ {
//                 continue 'rloop;
//             }
//         }
//         lhs.push_back((id, val));
//     }
//     lhs
// }


// fn change_binding<I: Debug + PartialEq, T>(mut bind: Vec<(I, T)>, id: &I, val: T) -> Result<Vec<(I, T)>, String> {
//     let mut good = false;
//     for id_, val_ in bind.iter_mut() {
//         if id == id_ {
//             *val_ = val;
//             good = true;
//             break;
//         }
//     }
//     if good { Ok(bind) } else { Err(format!("binding {:?} not found", id)) }
// }


// enum TypeVar<'a> {
//     Unknown,
//     Concrete(InternalType),
//     EqualTo(&'a TypeVar<'a>),
// }


// enum PartialIType<'a> {
//     Just(&'a TypeVar<'a>),
//     Func(Box<PartialIType<'a>>, Box<PartialIType<'a>>),
// }


// pub struct LocalBindings<'a> {
//     types: HashMap<usize, PartialIType>,
//     tvar_arena: TypedArena<TypeVar<'a>>,
// }

// impl<'a> LocalBindings<'a> {
//     pub fn new() -> LocalBindings<'a> {
//         LocalBindings {
//             types: HashMap::new(),
//             tvar_arena: TypedArena::new(),
//         }
//     }

//     pub fn term_type(&mut self, term: &Term, bindings: &mut Bindings) -> PartialIType<'a> {
//         match term {
//             Symbol(id) => if let Some(itype) = 
//         }
//     }
// }


// pub struct TermTypes {
//     term: Term,
//     itype: PartialIType,
//     vtypes: Vec<(usize, PartialIType)>,
// }

// impl TermTypes {
//     pub fn symbol(id: usize, bindings: &Bindings) -> TermTypes {
//         let known_type = bindings.get_term_type(id);
//         TermTypes {
//             term: Term::Symbol(id),
//             itype: if let Some(itype) = known_type { Complete(itype) } else { Unknown },
//             vtypes: if let Some(_) = known_type { vec![] } else { vec![(id, Unknown)] },
//         }
//     }

//     pub fn application(lhs: TermTypes, rhs: TermTypes, bindings: &Bindings) -> Result<TermTypes, String> {
//         match lhs.itype {
//             Complete(ltype) => if let Func(box lltype, box lrtype) = ltype {
//                 match rhs.itype {
//                     Complete(rtype)  => if lltype == rtype {
//                         Ok(TermTypes {
//                             term: Term::Application(Box::new(lhs.term), Box::new(rhs.term)),
//                             itype: Complete(lrtype),
//                             vtypes: concat_bindings(lhs.vtypes, rhs.vtypes),
//                         })
//                     } else { Err(format!("type mismatch : expected type {}, found type {}", lltype, rtype)) },
//                     FuncFrom(rltype) => if let Application(box Symbol(rlid), _) = rhs.term {
//                         Ok(TermTypes {
//                             term: Term::Application(Box::new(lhs.term), Box::new(rhs.term)),
//                             itype: Complete(lrtype),
//                             vtypes: change_binding(concat_bindings(lhs.vtypes, rhs.vtypes), &rlid, Complete(Func(Box::new(rltype), Box::new(lltype)))).unwrap(),
//                         })
//                     } else { Err(format!("i have no idea how i got here")) }
//                     Unknown          => if let Symbol(rid) = rhs.term {
//                         Ok(TermTypes {
//                             term: Term::Application(Box::new(lhs.term), Box::new(rhs.term)),
//                             itype: Complete(lrtype),
//                             vtypes: change_binding(concat_bindings(lhs.vtypes, rhs.vtypes), &rid, Complete(lltype)).unwrap(),
//                         })
//                     } else {
//                         Err(format!("this shouldn't happen"))
//                     }
//                 }
//             } else { Err(format!("cannot apply non-function term to another term")) }
//             FuncFrom(_)     => Err(format!("i think this is unreachable"))
//             Unknown         => match rhs.itype {
//                 Complete(rtype) => if let Symbol(lid) = lhs.term {
//                     Ok(TermTypes {
//                         term: Term::Application(Box::new(lhs.term), Box::new(rhs.term)),
//                         itype: Unknown,
//                         vtypes: change_binding(concat_bindings(lhs.vtypes, rhs.vtypes), &lid, FuncFrom(rtype)).unwrap(),
//                     })
//                 }
//             }
//         }
//     }
// }


// #[derive(Debug)]
// pub enum Expression {
//     Formula(Formula),
//     Schema(String, MetaType, Box<Expression>),
//     Generalization(usize, InternalType, Box<Expression>),
//     Assuming(Formula, Box<Expression>),
//     Contrapositive(Formula),
//     Block(StatementBlock),
// }


// #[derive(Debug)]
// pub enum Pattern {
//     Wildcard,
//     Symbol(usize),
//     Application(Box<Pattern>, usize),
// }


// #[derive(Debug)]
// pub enum Statement {
//     LetDec(String, MetaType),
//     LetEq(Pattern, Expression),
//     Axiom(Expression),
//     Proof(String, Expression, Expression)
//     Module(String, StatementBlock),
//     Import(Vec<String>, bool),
// }


// #[derive(Debug)]
// pub struct StatementBlock {
//     stmts: Vec<Statement>,
// }
