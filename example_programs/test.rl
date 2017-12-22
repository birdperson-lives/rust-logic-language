; This is a comment. Currently, comments may only occur at the beginning of a line.

let Nat :: Type
let zero :: Term Nat
let succ :: Term Nat -> Nat
let Eq :: Formula Nat Nat

axiom Reflexivity = forall x: Nat. Eq x x
axiom Symmetry = forall x: Nat. forall y: Nat. Eq x y -> Eq y x
axiom Transitivity = forall x: Nat. forall y: Nat. forall z: Nat. Eq x y -> Eq y z -> Eq x z

axiom SuccInjection = forall x: Nat. forall y: Nat. Eq (succ x) (succ y) -> Eq x y
axiom ZeroNotSucc = forall x: Nat. Eq (succ x) zero -> false

axiom Induction =
    schema Phi :: Formula Nat.
        Phi zero -> (forall x: Nat. Phi x -> Phi (succ x)) -> (forall x: Nat. Phi x)