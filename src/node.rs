//! The deferred layout tree behind the Python `Layout`.
//!
//! Composition does not build the native layout eagerly: it records the
//! child nodes and the native constructor to apply. The native constructors
//! take their operands by value, and Python `Layout` objects are shared and
//! immutable, so eager construction would have to clone every operand's
//! arena on each `+`/`&`/`@`/`pad(..)`/... call, making expression building
//! quadratic. Storing `Arc` children instead makes every composition O(1);
//! the native layout is materialized once, on demand, at `compile` or
//! `repr`.
//!
//! Both walks over the tree are iterative, like the upstream pipeline: a
//! layout as deep as Python cares to build materializes and frees in
//! constant native stack, with depth costing heap.

use std::sync::Arc;

use typeset::Layout;

/// A node: the constructor to apply, over its children in order. The
/// constructor fixes the arity: none for a leaf, one, two, or any number.
pub struct Node {
    op: Op,
    children: Vec<Arc<Node>>,
}

enum Op {
    /// A layout that is already built: `text`, `null`, and whole trees
    /// returned by `parse`.
    Leaf(Layout),
    Unary(fn(Layout) -> Layout),
    Binary(fn(Layout, Layout) -> Layout),
    Join(fn(Vec<Layout>) -> Layout),
}

impl Node {
    pub fn leaf(layout: Layout) -> Arc<Node> {
        Arc::new(Node {
            op: Op::Leaf(layout),
            children: Vec::new(),
        })
    }

    pub fn unary(build: fn(Layout) -> Layout, child: Arc<Node>) -> Arc<Node> {
        Arc::new(Node {
            op: Op::Unary(build),
            children: vec![child],
        })
    }

    pub fn binary(
        build: fn(Layout, Layout) -> Layout,
        left: Arc<Node>,
        right: Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node {
            op: Op::Binary(build),
            children: vec![left, right],
        })
    }

    pub fn join(build: fn(Vec<Layout>) -> Layout, children: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node {
            op: Op::Join(build),
            children,
        })
    }

    /// Build the native layout this node describes: a post-order walk with
    /// an explicit stack. A subtree reached through several parents is
    /// built once per parent, since each parent consumes its own copy.
    pub fn materialize(&self) -> Layout {
        enum Step<'a> {
            Enter(&'a Node),
            Build(&'a Node),
        }
        let mut steps = vec![Step::Enter(self)];
        let mut built: Vec<Layout> = Vec::new();
        while let Some(step) = steps.pop() {
            match step {
                Step::Enter(node) => {
                    steps.push(Step::Build(node));
                    steps.extend(node.children.iter().rev().map(|child| Step::Enter(child)));
                }
                Step::Build(node) => {
                    let layout = match node.op {
                        Op::Leaf(ref layout) => layout.clone(),
                        Op::Unary(build) => build(built.pop().expect("one child was built")),
                        Op::Binary(build) => {
                            let right = built.pop().expect("two children were built");
                            let left = built.pop().expect("two children were built");
                            build(left, right)
                        }
                        Op::Join(build) => {
                            build(built.split_off(built.len() - node.children.len()))
                        }
                    };
                    built.push(layout);
                }
            }
        }
        built.pop().expect("the root was built")
    }
}

/// Frees the tree with a worklist instead of the default field-by-field
/// drop, which would recurse once per level. A child still shared with
/// another parent is only released; one this node owned alone is unwrapped
/// and its children queued in turn.
impl Drop for Node {
    fn drop(&mut self) {
        let mut pending = std::mem::take(&mut self.children);
        while let Some(child) = pending.pop() {
            if let Ok(mut node) = Arc::try_unwrap(child) {
                pending.append(&mut node.children);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Node;
    use std::sync::Arc;
    use typeset::{nest, pad, text};

    /// Far deeper than the test thread's stack could recurse.
    const DEEP: usize = 1_000_000;

    #[test]
    fn deep_chains_materialize_and_drop_without_recursion() {
        let mut left_deep = Node::leaf(text("0"));
        let mut right_deep = Node::leaf(text("0"));
        let mut nested = Node::leaf(text("0"));
        for _ in 0..DEEP {
            left_deep = Node::binary(pad, left_deep, Node::leaf(text("w")));
            right_deep = Node::binary(pad, Node::leaf(text("w")), right_deep);
            nested = Node::unary(nest, nested);
        }
        let rendered = left_deep.materialize().compile().render(2, usize::MAX);
        assert_eq!(rendered.len(), 2 * DEEP + 1);
        let rendered = right_deep.materialize().compile().render(2, usize::MAX);
        assert_eq!(rendered.len(), 2 * DEEP + 1);
        // A nest indents the line it opens too: two columns per level.
        let rendered = nested.materialize().compile().render(2, 80);
        assert_eq!(rendered, format!("{}0", " ".repeat(2 * DEEP)));
        drop((left_deep, right_deep, nested));
    }

    #[test]
    fn shared_subtrees_are_built_per_parent_and_released_not_freed() {
        let shared = Node::leaf(text("x"));
        let both = Node::binary(pad, shared.clone(), shared.clone());
        assert_eq!(both.materialize().to_string(), r#""x" + "x""#);
        assert_eq!(Arc::strong_count(&shared), 3);
        drop(both);
        assert_eq!(Arc::strong_count(&shared), 1);
        assert_eq!(shared.materialize().to_string(), r#""x""#);
    }

    #[test]
    fn joins_take_their_children_in_order() {
        let items = (0..3).map(|i| Node::leaf(text(i.to_string()))).collect();
        let joined = Node::join(typeset::join_with_commas, items);
        assert_eq!(joined.materialize().compile().render(2, 80), "0, 1, 2");
        assert_eq!(
            Node::join(typeset::join_with_commas, Vec::new())
                .materialize()
                .to_string(),
            r#""""#
        );
    }
}
