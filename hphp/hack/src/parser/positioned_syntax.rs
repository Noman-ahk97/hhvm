/**
 * Copyright (c) 2019, Facebook, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the "hack" directory of this source tree.
 *
*/
use std::rc::Rc;

use crate::lexable_token::LexableToken;
use crate::positioned_token::PositionedToken;
use crate::syntax::{Syntax, SyntaxValueType, SyntaxVariant};
use crate::syntax_kind::SyntaxKind;
use crate::syntax_type::SyntaxType;
use crate::token_kind::TokenKind;

#[derive(Debug, Clone)]
pub enum PositionedValue {
    /// value for a token node is token itself
    TokenValue(PositionedToken),
    /// value for a range denoted by pair of tokens
    TokenSpan {
        left: PositionedToken,
        right: PositionedToken,
    },
    Missing {
        offset: usize,
    },
}

impl PositionedValue {
    pub fn width(&self) -> usize {
        match self {
            PositionedValue::TokenValue(t) => t.width(),
            PositionedValue::TokenSpan { left, right } => {
                (right.end_offset() - left.start_offset()) + 1
            }
            PositionedValue::Missing { .. } => 0,
        }
    }

    fn value_from_outer_children(
        first: &Syntax<PositionedToken, Self>,
        last: &Syntax<PositionedToken, Self>,
    ) -> Self {
        use PositionedValue::*;
        match (&first.value, &last.value) {
            (TokenValue(l), TokenValue(r))
            | (TokenSpan { left: l, .. }, TokenValue(r))
            | (TokenValue(l), TokenSpan { right: r, .. })
            | (TokenSpan { left: l, .. }, TokenSpan { right: r, .. }) => {
                if Rc::ptr_eq(&l.0, &r.0) {
                    TokenValue(PositionedToken::clone_rc(l))
                } else {
                    TokenSpan {
                        left: PositionedToken::clone_rc(l),
                        right: PositionedToken::clone_rc(r),
                    }
                }
            }
            // can have two missing nodes if first and last child nodes of
            // the node are missing - this means that entire node is missing.
            // NOTE: offset must match otherwise it will mean that there is a real node
            // in between that should be picked instead
            (Missing { offset: o1 }, Missing { offset: o2 }) if o1 == o2 => first.value.clone(),
            _ => panic!(),
        }
    }
}

impl SyntaxValueType<PositionedToken> for PositionedValue {
    fn from_syntax<'a>(variant: &'a SyntaxVariant<PositionedToken, Self>) -> Self {
        use PositionedValue::*;
        // We need to find the first and last nodes that are represented by tokens.
        // If there are no such nodes then we can simply use the first and last nodes, period,
        // since they will have an offset and source text we can use.
        let f = |node: &'a Syntax<PositionedToken, Self>,
                 (first, first_not_zero, last_not_zero, _last): (
            Option<&'a Syntax<PositionedToken, Self>>,
            Option<&'a Syntax<PositionedToken, Self>>,
            Option<&'a Syntax<PositionedToken, Self>>,
            Option<&'a Syntax<PositionedToken, Self>>,
        )| {
            match (first.is_some(), first_not_zero.is_some(), &node.value) {
                (false, false, TokenValue { .. }) | (false, false, TokenSpan { .. }) => {
                    // first iteration and first node has some token representation -
                    // record it as first, first_non_zero, last and last_non_zero
                    (Some(node), Some(node), Some(node), Some(node))
                }
                (false, false, _) => {
                    // first iteration - first node is missing -
                    // record it as first and last
                    (Some(node), None, None, Some(node))
                }
                (true, false, TokenValue { .. }) | (true, false, TokenSpan { .. }) => {
                    // in progress, found first node that include tokens -
                    // record it as first_non_zero, last and last_non_zero
                    (first, Some(node), Some(node), Some(node))
                }
                (true, true, TokenValue { .. }) | (true, true, TokenSpan { .. }) => {
                    // in progress found Some (node) that include tokens -
                    // record it as last_non_zero and last
                    (first, first_not_zero, Some(node), Some(node))
                }
                _ => {
                    // in progress, stepped on missing node -
                    // record it as last and move on
                    ((first, first_not_zero, last_not_zero, Some(node)))
                }
            }
        };
        let (f, fnz, lnz, l) =
            SyntaxType::fold_over_children(&f, (None, None, None, None), variant);
        match (f, fnz, lnz, l) {
            (_, Some(first_not_zero), Some(last_not_zero), _) => {
                Self::value_from_outer_children(first_not_zero, last_not_zero)
            }
            (Some(first), None, None, Some(last)) => Self::value_from_outer_children(first, last),
            _ => panic!("how did we get a node with no children in value_from_syntax?"),
        }
    }

    fn from_children(
        _: SyntaxKind,
        offset: usize,
        nodes: &[Syntax<PositionedToken, Self>],
    ) -> Self {
        // We need to determine the offset, leading, middle and trailing widths of
        // the node to be constructed based on its children.  If the children are
        // all of zero width -- including the case where there are no children at
        // all -- then we make a zero-width value at the given offset.
        // Otherwise, we can determine the associated value from the first and last
        // children that have width.
        let mut have_width = nodes.iter().filter(|x| x.value.width() > 0).peekable();
        match have_width.peek() {
            None => PositionedValue::Missing { offset },
            Some(first) => Self::value_from_outer_children(first, have_width.last().unwrap()),
        }
    }

    fn from_token(token: &PositionedToken) -> Self {
        if token.kind() == TokenKind::EndOfFile || token.full_width() == 0 {
            PositionedValue::Missing {
                offset: token.end_offset(),
            }
        } else {
            PositionedValue::TokenValue(PositionedToken::clone_rc(token))
        }
    }
}

pub type PositionedSyntax = Syntax<PositionedToken, PositionedValue>;
