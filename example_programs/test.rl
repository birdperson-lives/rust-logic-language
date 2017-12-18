; This is a comment. Currently, comments may only occur at the beginning of a line.

let Nat :: Type
let Zero :: Term Nat
let Succ :: Term Nat -> Nat
let Eq :: Formula Nat Nat

axiom Reflexivity = forall x: Nat. Eq x x
axiom Symmetry = forall x: Nat. forall y: Nat. Eq x y -> Eq y x
axiom Transitivity = forall x: Nat. forall y: Nat. forall z: Nat. Eq x y -> Eq y z -> Eq x z

axiom SuccInjection = forall x: Nat. forall y: Nat. Eq (Succ x) (Succ y) -> Eq x y
axiom ZeroNotSucc = forall x: Nat. Eq (Succ x) Zero -> false

axiom Induction =
    schema phi :: Formula Nat.
        phi Zero -> (forall x: Nat. phi x -> phi (Succ x)) -> (forall x: Nat. phi x)