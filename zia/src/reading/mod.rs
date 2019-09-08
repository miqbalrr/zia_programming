/*  Library for the Zia programming language.
    Copyright (C) 2018 to 2019 Charles Johnson

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program. If not, see <http://www.gnu.org/licenses/>.
*/

mod concepts;
mod syntax;

pub use self::concepts::*;
pub use self::syntax::*;
use constants::{ASSOC, FALSE, LABEL, LEFT, PRECEDENCE, REDUCTION, RIGHT, TRUE};
use delta::Delta;
use std::{collections::HashSet, fmt, rc::Rc, str::FromStr};

pub trait SyntaxReader<T, U>
where
    Self: GetLabel<T> + FindDefinition<T>,
    T: GetDefinitionOf + GetDefinition + GetReduction + MaybeString + fmt::Debug,
    U: FromStr
        + BindConcept
        + Clone
        + BindPair
        + MaybeConcept
        + MightExpand
        + fmt::Display
        + PartialEq,
    <U as FromStr>::Err: fmt::Debug,
{
    /// Expands syntax by definition of its associated concept.
    fn expand(&self, deltas: &[Self::Delta], ast: &Rc<U>) -> Rc<U> {
        if let Some(con) = ast.get_concept() {
            if let Some((left, right)) = self.read_concept(deltas, con).get_definition() {
                self.combine(
                    deltas,
                    &self.expand(deltas, &self.to_ast(deltas, left)),
                    &self.expand(deltas, &self.to_ast(deltas, right)),
                )
            } else {
                self.to_ast(deltas, con)
            }
        } else if let Some((ref left, ref right)) = ast.get_expansion() {
            self.combine(
                deltas,
                &self.expand(deltas, left),
                &self.expand(deltas, right),
            )
        } else {
            ast.clone()
        }
    }
    /// Reduces the syntax as much as possible (returns the normal form syntax).
    fn recursively_reduce(&self, deltas: &[Self::Delta], ast: &Rc<U>) -> Rc<U> {
        match self.reduce(deltas, ast) {
            Some(ref a) => self.recursively_reduce(deltas, a),
            None => ast.clone(),
        }
    }
    fn determine_reduction_truth(
        &self,
        deltas: &[Self::Delta],
        left: &Rc<U>,
        right: &Rc<U>,
    ) -> Option<bool> {
        if left == right {
            Some(false)
        } else {
            self.determine_evidence_of_reduction(deltas, left, right)
                .or_else(|| {
                    self.determine_evidence_of_reduction(deltas, right, left)
                        .map(|x| !x)
                })
        }
    }
    fn determine_evidence_of_reduction(
        &self,
        deltas: &[Self::Delta],
        left: &Rc<U>,
        right: &Rc<U>,
    ) -> Option<bool> {
        self.reduce(deltas, left).and_then(|reduced_left| {
            if &reduced_left == right {
                Some(true)
            } else {
                self.determine_evidence_of_reduction(deltas, &reduced_left, right)
            }
        })
    }
    /// Reduces the syntax by using the reduction rules of associated concepts.
    fn reduce(&self, deltas: &[Self::Delta], ast: &Rc<U>) -> Option<Rc<U>> {
        match ast.get_concept() {
            Some(c) => self.reduce_concept(deltas, c),
            None => ast.get_expansion().and_then(|(ref left, ref right)| {
                left.get_concept()
                    .and_then(|lc| match lc {
                        ASSOC => Some(
                            self.to_ast(deltas, RIGHT)
                        ),
                        _ => None,
                    })
                    .or_else(|| {
                        right
                            .get_expansion()
                            .and_then(|(rightleft, rightright)| {
                                rightleft.get_concept().and_then(|rlc| match rlc {
                                    REDUCTION => self
                                        .determine_reduction_truth(deltas, left, &rightright)
                                        .map(|x| {
                                            if x {
                                                self.to_ast(deltas, TRUE)
                                            } else {
                                                self.to_ast(deltas, FALSE)
                                            }
                                        }),
                                    _ => None,
                                })
                            })
                            .or_else(|| {
                                self.match_left_right(
                                    deltas,
                                    self.reduce(deltas, left),
                                    self.reduce(deltas, right),
                                    left,
                                    right,
                                )
                            })
                    })
            }),
        }
    }
    /// Returns the syntax for the reduction of a concept.
    fn reduce_concept(&self, deltas: &[Self::Delta], concept: usize) -> Option<Rc<U>> {
        self.read_concept(deltas, concept)
            .get_reduction()
            .map(|n| self.to_ast(deltas, n))
            .or_else(|| {
                self.read_concept(deltas, concept)
                    .get_definition()
                    .and_then(|(left, right)| {
                        let left_result = self.reduce_concept(deltas, left);
                        let right_result = self.reduce_concept(deltas, right);
                        self.match_left_right(
                            deltas,
                            left_result,
                            right_result,
                            &self.to_ast(deltas, left),
                            &self.to_ast(deltas, right),
                        )
                    })
            })
    }
    /// Returns the syntax for a concept.
    fn to_ast(&self, deltas: &[Self::Delta], concept: usize) -> Rc<U> {
        match self.get_label(deltas, concept) {
            Some(s) => Rc::new(s.parse::<U>().unwrap().bind_concept(concept)),
            None => {
                let (left, right) = self
                    .read_concept(deltas, concept)
                    .get_definition()
                    .expect("Unlabelled concept with no definition");
                self.combine(
                    deltas,
                    &self.to_ast(deltas, left),
                    &self.to_ast(deltas, right),
                )
            }
        }
    }
    fn combine(&self, deltas: &[Self::Delta], ast: &Rc<U>, other: &Rc<U>) -> Rc<U> {
        let syntax = ast
            .get_concept()
            .and_then(|l| {
                other.get_concept().and_then(|r| {
                    self.find_definition(deltas, l, r)
                        .map(|concept| self.join(deltas, ast, other).bind_concept(concept))
                })
            })
            .unwrap_or_else(|| self.join(deltas, ast, other));
        Rc::new(syntax)
    }
    fn join(&self, deltas: &[Self::Delta], left: &Rc<U>, right: &Rc<U>) -> U {
        self.display_joint(deltas, left, right)
            .parse::<U>()
            .unwrap()
            .bind_pair(left, right)
    }
    fn display_joint(&self, deltas: &[Self::Delta], left: &Rc<U>, right: &Rc<U>) -> String {
        let left_string = left
            .get_expansion()
            .map(|(l, r)| match self.get_associativity(deltas, &r).unwrap() {
                Associativity::Left => l.to_string() + " " + &r.to_string(),
                Associativity::Right => {
                    "(".to_string() + &l.to_string() + " " + &r.to_string() + ")"
                }
            })
            .unwrap_or_else(|| left.to_string());
        let right_string = right
            .get_expansion()
            .map(|(l, r)| match self.get_associativity(deltas, &l).unwrap() {
                Associativity::Left => {
                    "(".to_string() + &l.to_string() + " " + &r.to_string() + ")"
                }
                Associativity::Right => l.to_string() + " " + &r.to_string(),
            })
            .unwrap_or_else(|| right.to_string());
        left_string + " " + &right_string
    }
    fn get_associativity(&self, deltas: &[Self::Delta], ast: &Rc<U>) -> Option<Associativity> {
        let assoc_of_ast = self.combine(deltas, &self.to_ast(deltas, ASSOC), &ast);
        self.reduce(deltas, &assoc_of_ast)
            .and_then(|ast| match ast.get_concept() {
                Some(LEFT) => Some(Associativity::Left),
                Some(RIGHT) => Some(Associativity::Right),
                _ => None,
            })
    }
    fn has_higher_precedence(
        &self,
        deltas: &[Self::Delta],
        left: &Rc<U>,
        right: &Rc<U>,
    ) -> Option<bool> {
        let is_higher_prec_than_right =
            self.combine(deltas, &self.to_ast(deltas, PRECEDENCE), &right);
        let left_is_higher_prec_than_right = self.combine(deltas, left, &is_higher_prec_than_right);
        self.reduce(deltas, &left_is_higher_prec_than_right)
            .and_then(|ast| match ast.get_concept() {
                Some(TRUE) => Some(true),
                Some(FALSE) => Some(false),
                _ => None,
            })
    }
    /// Returns the updated branch of abstract syntax tree that may have had the left or right parts updated.
    fn match_left_right(
        &self,
        deltas: &[Self::Delta],
        left: Option<Rc<U>>,
        right: Option<Rc<U>>,
        original_left: &Rc<U>,
        original_right: &Rc<U>,
    ) -> Option<Rc<U>> {
        match (left, right) {
            (None, None) => None,
            (Some(new_left), None) => Some(self.contract_pair(deltas, &new_left, original_right)),
            (None, Some(new_right)) => Some(self.contract_pair(deltas, original_left, &new_right)),
            (Some(new_left), Some(new_right)) => {
                Some(self.contract_pair(deltas, &new_left, &new_right))
            }
        }
    }
    /// Returns the abstract syntax from two syntax parts, using the label and concept of the composition of associated concepts if it exists.
    fn contract_pair(&self, deltas: &[Self::Delta], lefthand: &Rc<U>, righthand: &Rc<U>) -> Rc<U> {
        Rc::new(
            lefthand
                .get_concept()
                .and_then(|lc| {
                    righthand.get_concept().and_then(|rc| {
                        self.find_definition(deltas, lc, rc).and_then(|def| {
                            self.get_label(deltas, def)
                                .map(|label| label.parse::<U>().unwrap().bind_concept(def))
                        })
                    })
                })
                .unwrap_or_else(|| {
                    self.display_joint(deltas, lefthand, righthand)
                        .parse::<U>()
                        .unwrap()
                })
                .bind_pair(lefthand, righthand),
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum Associativity {
    Left,
    Right,
}

impl<S, T, U> SyntaxReader<T, U> for S
where
    S: GetLabel<T>,
    T: GetDefinitionOf + GetDefinition + MaybeString + GetReduction + fmt::Debug,
    U: FromStr
        + BindConcept
        + Clone
        + BindPair
        + MaybeConcept
        + MightExpand
        + fmt::Display
        + PartialEq,
    <U as FromStr>::Err: fmt::Debug,
{
}

pub trait GetLabel<T>
where
    T: MaybeString + GetDefinitionOf + GetDefinition + GetReduction + fmt::Debug,
    Self: GetNormalForm<T> + GetConceptOfLabel<T>,
{
    fn get_label(&self, deltas: &[Self::Delta], concept: usize) -> Option<String> {
        match self.get_concept_of_label(deltas, concept) {
            None => self
                .read_concept(deltas, concept)
                .get_reduction()
                .and_then(|r| self.get_label(deltas, r)),
            Some(d) => self
                .get_normal_form(deltas, d)
                .and_then(|n| self.read_concept(deltas, n).get_string()),
        }
    }
}

impl<S, T> GetLabel<T> for S
where
    T: MaybeString + GetDefinitionOf + GetDefinition + GetReduction + fmt::Debug,
    S: GetNormalForm<T> + GetConceptOfLabel<T>,
{
}

pub trait Label<T>
where
    T: GetDefinition + FindWhatReducesToIt,
    Self: FindWhatItsANormalFormOf<T>,
{
    fn get_labellee(&self, deltas: &[Self::Delta], concept: usize) -> Option<usize> {
        let mut candidates: Vec<usize> = Vec::new();
        for label in self.find_what_its_a_normal_form_of(deltas, concept) {
            match self.read_concept(deltas, label).get_definition() {
                None => continue,
                Some((r, x)) => {
                    if r == LABEL {
                        candidates.push(x)
                    } else {
                        continue;
                    }
                }
            };
        }
        match candidates.len() {
            0 => None,
            1 => Some(candidates[0]),
            _ => panic!("Multiple concepts are labelled with the same string"),
        }
    }
}

impl<S, T> Label<T> for S
where
    S: FindWhatItsANormalFormOf<T>,
    T: GetDefinition + FindWhatReducesToIt,
{
}
pub trait GetNormalForm<T>
where
    T: GetReduction,
    Self: ConceptReader<T> + Delta,
{
    fn get_normal_form(&self, deltas: &[Self::Delta], concept: usize) -> Option<usize> {
        self.read_concept(deltas, concept)
            .get_reduction()
            .map(|n| self.get_normal_form(deltas, n).unwrap_or(n))
    }
}

impl<S, T> GetNormalForm<T> for S
where
    S: ConceptReader<T>,
    T: GetReduction,
{
}

pub trait GetConceptOfLabel<T>
where
    T: GetDefinition + GetDefinitionOf + fmt::Debug,
    Self: ConceptReader<T>,
{
    fn get_concept_of_label(&self, deltas: &[Self::Delta], concept: usize) -> Option<usize> {
        self.read_concept(deltas, concept)
            .get_righthand_of()
            .iter()
            .filter(|candidate| {
                self.read_concept(deltas, **candidate)
                    .get_definition()
                    .expect("Candidate should have a definition!")
                    .0
                    == LABEL
            })
            .nth(0)
            .cloned()
    }
}

impl<S, T> GetConceptOfLabel<T> for S
where
    T: GetDefinition + GetDefinitionOf + fmt::Debug,
    S: ConceptReader<T>,
{
}

pub trait MaybeDisconnected<T>
where
    T: GetReduction + FindWhatReducesToIt + GetDefinition + GetDefinitionOf,
    Self: ConceptReader<T>,
{
    fn is_disconnected(&self, deltas: &[Self::Delta], concept: usize) -> bool {
        self.read_concept(deltas, concept).get_reduction().is_none()
            && self
                .read_concept(deltas, concept)
                .get_definition()
                .is_none()
            && self
                .read_concept(deltas, concept)
                .get_lefthand_of()
                .is_empty()
            && self.righthand_of_without_label_is_empty(deltas, concept)
            && self
                .read_concept(deltas, concept)
                .find_what_reduces_to_it()
                .is_empty()
    }
    fn righthand_of_without_label_is_empty(&self, deltas: &[Self::Delta], con: usize) -> bool {
        self.read_concept(deltas, con)
            .get_righthand_of()
            .iter()
            .filter_map(|concept| {
                self.read_concept(deltas, *concept)
                    .get_definition()
                    .filter(|(left, _)| *left != LABEL)
            })
            .nth(0)
            .is_none()
    }
}

impl<S, T> MaybeDisconnected<T> for S
where
    T: GetReduction + FindWhatReducesToIt + GetDefinition + GetDefinitionOf,
    S: ConceptReader<T>,
{
}

pub trait FindDefinition<T>
where
    T: GetDefinitionOf,
    Self: ConceptReader<T>,
{
    fn find_definition(
        &self,
        deltas: &[Self::Delta],
        lefthand: usize,
        righthand: usize,
    ) -> Option<usize> {
        let has_lefthand = self.read_concept(deltas, lefthand).get_lefthand_of();
        let has_righthand = self.read_concept(deltas, righthand).get_righthand_of();
        let mut candidates = has_lefthand.intersection(&has_righthand);
        candidates.next().map(|index| {
            candidates.next().map_or(*index, |_| {
                panic!("Multiple definitions with the same lefthand and righthand pair exist.")
            })
        })
    }
}

impl<S, T> FindDefinition<T> for S
where
    T: GetDefinitionOf,
    S: ConceptReader<T>,
{
}

pub trait FindWhatItsANormalFormOf<T>
where
    T: FindWhatReducesToIt,
    Self: ConceptReader<T> + Delta,
{
    fn find_what_its_a_normal_form_of(&self, deltas: &[Self::Delta], con: usize) -> HashSet<usize> {
        let mut normal_form_of = self.read_concept(deltas, con).find_what_reduces_to_it();
        for concept in normal_form_of.clone().iter() {
            for concept2 in self.find_what_its_a_normal_form_of(deltas, *concept).iter() {
                normal_form_of.insert(*concept2);
            }
        }
        normal_form_of
    }
}

impl<S, T> FindWhatItsANormalFormOf<T> for S
where
    S: ConceptReader<T>,
    T: FindWhatReducesToIt,
{
}

pub trait Container<T>
where
    Self: ConceptReader<T>,
    T: GetDefinition,
{
    fn contains(&self, deltas: &[Self::Delta], outer: usize, inner: usize) -> bool {
        if let Some((left, right)) = self.read_concept(deltas, outer).get_definition() {
            left == inner
                || right == inner
                || self.contains(deltas, left, inner)
                || self.contains(deltas, right, inner)
        } else {
            false
        }
    }
}

impl<S, T> Container<T> for S
where
    S: ConceptReader<T>,
    T: GetDefinition,
{
}

pub trait ConceptReader<T>
where
    Self: Delta,
{
    fn read_concept(&self, &[Self::Delta], usize) -> T;
}

pub trait BindConcept {
    fn bind_concept(self, usize) -> Self;
}
