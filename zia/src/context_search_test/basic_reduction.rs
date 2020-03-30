use crate::{
    ast::SyntaxTree,
    concepts::{Concept, SpecificPart},
    context_delta::ContextDelta,
    context_search::{ContextCache, ContextSearch},
    snap_shot::Reader as SnapShotReader,
};
use lazy_static::lazy_static;
use std::sync::Arc;

#[derive(Default)]
struct BasicReductionSnapShot;

lazy_static! {
    static ref CONCEPTS: (Concept, Concept) = {
        let mut concrete_concept = (SpecificPart::Concrete, 0).into();
        let mut abstract_concept: Concept = (SpecificPart::default(), 1).into();
        abstract_concept.make_reduce_to(&mut concrete_concept);
        (concrete_concept, abstract_concept)
    };
    static ref CONCRETE_SYNTAX: SyntaxTree =
        SyntaxTree::from("concrete").bind_concept(0);
    static ref ABSTRACT_SYNTAX: SyntaxTree =
        SyntaxTree::from("abstract").bind_concept(1);
}

impl SnapShotReader for BasicReductionSnapShot {
    fn read_concept(
        &self,
        _delta: &ContextDelta,
        concept_id: usize,
    ) -> Concept {
        let (concrete_concept, abstract_concept) = CONCEPTS.clone();
        match concept_id {
            0 => concrete_concept,
            1 => abstract_concept,
            _ => panic!("No concepts with id: {}", concept_id),
        }
    }

    fn true_id() -> usize {
        unimplemented!()
    }

    fn implication_id() -> usize {
        unimplemented!()
    }

    fn precedence_id() -> usize {
        unimplemented!()
    }

    fn greater_than_id() -> usize {
        unimplemented!()
    }

    fn default_id() -> usize {
        unimplemented!()
    }

    fn false_id() -> usize {
        unimplemented!()
    }

    fn reduction_id() -> usize {
        unimplemented!()
    }

    fn assoc_id() -> usize {
        unimplemented!()
    }

    fn right_id() -> usize {
        unimplemented!()
    }

    fn left_id() -> usize {
        unimplemented!()
    }

    fn exists_such_that_id() -> usize {
        unimplemented!()
    }

    fn concept_from_label(
        &self,
        _: &ContextDelta,
        _label: &str,
    ) -> Option<usize> {
        unimplemented!()
    }

    fn has_variable(&self, _delta: &ContextDelta, _variable_id: usize) -> bool {
        false
    }

    fn concept_len(&self, _delta: &ContextDelta) -> usize {
        2
    }

    fn get_label(
        &self,
        _delta: &ContextDelta,
        concept_id: usize,
    ) -> Option<String> {
        match concept_id {
            0 => Some("concrete".into()),
            1 => Some("abstract".into()),
            _ => None,
        }
    }

    fn ast_from_symbol(
        &self,
        _delta: &ContextDelta,
        symbol: &str,
    ) -> SyntaxTree {
        match symbol {
            "concrete" => CONCRETE_SYNTAX.clone(),
            "abstract" => ABSTRACT_SYNTAX.clone(),
            _ => symbol.into(),
        }
    }
}

#[test]
fn basic_reduction() {
    let snapshot = BasicReductionSnapShot::default();
    let delta = ContextDelta::default();
    let cache = ContextCache::default();
    let context_search = ContextSearch::<BasicReductionSnapShot>::from((
        &snapshot, &delta, &cache,
    ));
    let abstract_syntax = Arc::new(ABSTRACT_SYNTAX.clone());
    let concrete_syntax = Arc::new(CONCRETE_SYNTAX.clone());

    assert_eq!(
        context_search.recursively_reduce(&abstract_syntax),
        concrete_syntax
    );
    assert_eq!(
        context_search.recursively_reduce(&concrete_syntax),
        concrete_syntax
    );

    assert_eq!(
        context_search.reduce(&abstract_syntax),
        Some(concrete_syntax.clone())
    );
    assert_eq!(context_search.reduce(&concrete_syntax), None);

    assert_eq!(
        context_search.ast_from_expression("abstract"),
        Ok(abstract_syntax.clone())
    );
    assert_eq!(
        context_search.ast_from_expression("concrete"),
        Ok(concrete_syntax.clone())
    );

    assert_eq!(context_search.expand(&abstract_syntax), abstract_syntax);
    assert_eq!(context_search.expand(&concrete_syntax), concrete_syntax);

    assert_eq!(context_search.to_ast(0), concrete_syntax);
    assert_eq!(context_search.to_ast(1), abstract_syntax);
}
