#![allow(dead_code)]

use std::boxed::Box;
// use std::cmp::{PartialEq, Eq};
use std::collections::HashMap;
// use std::fmt;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ident {
    Global(usize),
    Local(usize),
}

impl Ident {
    fn matches(self, other: Ident, assoc_sto: &mut HashMap<usize, usize>, assoc_ots: &mut HashMap<usize, usize>) -> bool {
        match self {
            Ident::Global(id)      => if let Ident::Global(id_) = other { id == id_ } else { false },
            Ident::Local(local_id) => {
                if let Ident::Local(local_id_) = other {
                    *assoc_sto.entry(local_id).or_insert(local_id_) == local_id_ &&
                        *assoc_ots.entry(local_id_).or_insert(local_id) == local_id
                } else {
                    false
                }
            },
        }
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InternalType {
    Named(Ident),
    Func(Box<InternalType>, Box<InternalType>),
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaType {
    Type,
    Term(InternalType),
    Formula(Vec<InternalType>),
    Schema(Vec<MetaType>, Box<MetaType>),
}


#[derive(Clone, Debug)]
pub enum MetaValue {
    Type(InternalType),
    Term(Term),
    Formula(Formula),
    Schema(FormulaSchema),
}


#[derive(Clone, Debug)]
pub enum Term {
    Symbol(Ident),
    Application(Box<Term>, Box<Term>),
}

impl Term {
    fn substitute(self, var: usize, term: &Term) -> Term {
        match self {
            Term::Symbol(id)                     => if id == Ident::Local(var) { term.clone() } else { Term::Symbol(id) },
            Term::Application(box func, box arg) => Term::Application(Box::new(func.substitute(var, term)), Box::new(arg.substitute(var, term))),
        }
    }

    // pub fn vars(&self) -> HashSet<usize> {
    //     match self {
    //         &Term::Variable(name)                 => {
    //             let mut result = HashSet::new();
    //             result.insert(name.clone());
    //             result
    //         },
    //         &Term::Application(box func, box arg) => &func.vars() | &arg.vars(),
    //         // _                                     => HashSet::new(),
    //     }
    // }

    fn matches(&self, other: &Term, assoc_sto: &mut HashMap<usize, usize>, assoc_ots: &mut HashMap<usize, usize>) -> bool {
        match self {
            &Term::Symbol(id)                             => {
                if let &Term::Symbol(id_) = other {
                    Ident::matches(id, id_, assoc_sto, assoc_ots)
                } else {
                    false
                }
            },
            &Term::Application(box ref func, box ref arg) => {
                if let &Term::Application(box ref func_, box ref arg_) = other {
                    Term::matches(func, func_, assoc_sto, assoc_ots) && Term::matches(arg, arg_, assoc_sto, assoc_ots)
                } else {
                    false
                }
            },
        }
    }
}


#[derive(Clone, Debug)]
pub enum Formula {
    False,
    Relation(Ident),
    Application(Box<Formula>, Term),
    Implication(Box<Formula>, Box<Formula>),
    UniversalQ(usize, InternalType, Box<Formula>),
    // ExistentialQ(usize, InternalType, Box<Formula>),
}

impl Formula {
    // pub fn negate(self) -> Formula {
    //     Formula::Implication(Box::new(self), Box::new(Formula::False))
    // }

    // pub fn contrapositive(self) -> Formula {
    //     if let Formula::Implication(box lhs, box rhs) = self {
    //         Formula::Implication(Box::new(rhs.negate()), Box::new(lhs.negate()))
    //     } else {
    //         panic!("Cannot take contrapositive of {:?} : not an implication", self);
    //     }
    // }

    pub fn instantiate(self, term: Term) -> Formula {
        if let Formula::UniversalQ(var, _itype, box form) = self {
            form.substitute(var, &term)
        } else {
            panic!("Cannot instantiate in {:?} : not a universal quantification", self);
        }
    }

    pub fn substitute(self, var: usize, term: &Term) -> Formula {
        match self {
            Formula::False                           => Formula::False,
            Formula::Relation(id)                    => Formula::Relation(id),
            Formula::Application(box pred, arg)      => Formula::Application(Box::new(pred.substitute(var, term)), arg.substitute(var, term)),
            Formula::Implication(box lhs, box rhs)   => Formula::Implication(Box::new(lhs.substitute(var, term)), Box::new(rhs.substitute(var, term))),
            Formula::UniversalQ(id, itype, box form) => {
                if id == var {
                    Formula::UniversalQ(id, itype, Box::new(form))
                } else {
                    Formula::UniversalQ(id, itype, Box::new(form.substitute(var, term)))
                }
            },
        }
    }

    pub fn matches(&self, other: &Formula, assoc_sto: &mut HashMap<usize, usize>, assoc_ots: &mut HashMap<usize, usize>) -> bool {
        match self {
            &Formula::False                                   => if let &Formula::False = other { true } else { false },
            &Formula::Relation(id)                            => {
                if let &Formula::Relation(id_) = other {
                    Ident::matches(id, id_, assoc_sto, assoc_ots)
                } else {
                    false
                }
            },
            &Formula::Application(box ref pred, ref arg)      => {
                if let &Formula::Application(box ref pred_, ref arg_) = other {
                    Formula::matches(pred, pred_, assoc_sto, assoc_ots) && Term::matches(arg, arg_, assoc_sto, assoc_ots)
                } else {
                    false
                }
            },
            &Formula::Implication(box ref lhs, box ref rhs)   => {
                if let &Formula::Implication(box ref lhs_, box ref rhs_) = other {
                    Formula::matches(lhs, lhs_, assoc_sto, assoc_ots) && Formula::matches(rhs, rhs_, assoc_sto, assoc_ots)
                } else {
                    false
                }
            },
            &Formula::UniversalQ(id, ref itype, box ref form) => {
                if let &Formula::UniversalQ(id_, ref itype_, box ref form_) = other {
                    assoc_sto.insert(id, id_);
                    assoc_ots.insert(id_, id);
                    let answer = itype == itype_ && Formula::matches(form, form_, assoc_sto, assoc_ots);
                    assoc_sto.remove(&id);
                    assoc_ots.remove(&id_);
                    answer
                } else {
                    false
                }
            },
        }
    }

    // fn free_vars(&self) -> HashSet<usize> {
    //     match self {
    //         &Formula::Application(box pred, term)   => &pred.free_vars() | &term.vars(),
    //         &Formula::Implication(box lhs, box rhs) => &lhs.free_vars() | &rhs.free_vars(),
    //         &Formula::UniversalQ(var, box content)  => {
    //             let mut result = content.free_vars();
    //             result.remove(&var);
    //             result
    //         },
    //         _                                       => HashSet::new(),
    //     }
    // }

    // pub fn is_wff(&self) -> bool {
    //     self.free_vars().len() == 0
    // }
}


#[derive(Clone, Debug)]
pub enum FormulaSchema {
    Formula(Formula),
    Schema(usize, MetaType, Box<FormulaSchema>),
}

impl FormulaSchema {
    // pub fn specify(self, )
}