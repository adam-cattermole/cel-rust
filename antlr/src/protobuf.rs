#[cfg(feature = "protobuf")]
pub mod protobuf {
    pub mod types {
        include!(concat!(env!("OUT_DIR"), "/cel.expr.rs"));
    }

    pub use types::*;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_protobuf_types_accessible() {
            let _expr = Expr {
                id: 1,
                expr_kind: None,
            };

            let _constant = Constant {
                constant_kind: Some(constant::ConstantKind::BoolValue(true)),
            };

            let _parsed_expr = ParsedExpr {
                expr: None,
                source_info: None,
            };
        }
    }
}
