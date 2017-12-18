#![allow(dead_code)]

use std::boxed::Box;
// use std::cmp::{PartialEq, Eq};
// use std::collections::HashSet;
// use std::fmt;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ident {
    Local(usize),
    Global(usize),
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
    pub fn negate(self) -> Formula {
        Formula::Implication(Box::new(self), Box::new(Formula::False))
    }

    pub fn contrapositive(self) -> Formula {
        if let Formula::Implication(box lhs, box rhs) = self {
            Formula::Implication(Box::new(rhs.negate()), Box::new(lhs.negate()))
        } else {
            panic!("Cannot take contrapositive of {:?} : not an implication", self);
        }
    }

    pub fn instantiate(self, term: Term) -> Formula {
        if let Formula::UniversalQ(var, _itype, box form) = self {
            form.substitute(var, &term)
        } else {
            panic!("Cannot instantiate in {:?} : not a universal quantification", self);
        }
    }

    fn substitute(self, var: usize, term: &Term) -> Formula {
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