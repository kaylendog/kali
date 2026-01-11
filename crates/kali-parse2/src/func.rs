use chumsky::prelude::*;
use kali_ast2::{FnItem, FnParam};

use crate::{define_parser, ident};

define_parser!(fn_item: FnItem, {}, {
    just("fn")
        .ignore_then(ident())
        .then(fn_param().repeated().collect())
        .map_with(|(name, parameters), e| FnItem {
            id: 0,
            name,
            parameters,
            span: e.span(),
        })
});

define_parser!(fn_param: FnParam, {}, {
    ident().map_with(|name, e| FnParam {
        id: 0,
        name,
        ty: None,
        span: e.span(),
    })
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_parse_err, assert_parse_ok, span_of};
    use chumsky::Parser;
}
