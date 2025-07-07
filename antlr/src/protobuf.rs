#[cfg(feature = "protobuf")]
pub mod protobuf {
    pub mod types {
        include!(concat!(env!("OUT_DIR"), "/cel.expr.rs"));
    }

    pub use types::*;

    use crate::ast::{Expr as AstExpr, IdedExpr};
    use crate::reference::Val;

    #[derive(Debug, Clone, PartialEq)]
    pub enum ConversionError {
        MissingField { field: String },
        InvalidExpressionKind,
        InvalidConstantKind,
        InvalidId(String),
    }

    impl std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ConversionError::MissingField { field } => {
                    write!(f, "Missing required field: {}", field)
                }
                ConversionError::InvalidExpressionKind => {
                    write!(f, "Invalid expression kind")
                }
                ConversionError::InvalidConstantKind => {
                    write!(f, "Invalid constant kind")
                }
                ConversionError::InvalidId(id) => {
                    write!(f, "Invalid ID: {}", id)
                }
            }
        }
    }

    impl std::error::Error for ConversionError {}

    impl From<Val> for Constant {
        fn from(val: Val) -> Self {
            let constant_kind = match val {
                Val::Null => Some(constant::ConstantKind::NullValue(0)),
                Val::Boolean(b) => Some(constant::ConstantKind::BoolValue(b)),
                Val::Int(i) => Some(constant::ConstantKind::Int64Value(i)),
                Val::UInt(i) => Some(constant::ConstantKind::Uint64Value(i)),
                Val::Double(d) => Some(constant::ConstantKind::DoubleValue(d)),
                Val::String(s) => Some(constant::ConstantKind::StringValue(s)),
                Val::Bytes(b) => Some(constant::ConstantKind::BytesValue(b)),
            };
            Constant { constant_kind }
        }
    }

    impl TryFrom<Constant> for Val {
        type Error = ConversionError;

        fn try_from(constant: Constant) -> Result<Self, Self::Error> {
            match constant.constant_kind {
                None => Err(ConversionError::MissingField {
                    field: "constant_kind".to_string(),
                }),
                Some(constant::ConstantKind::NullValue(_)) => Ok(Val::Null),
                Some(constant::ConstantKind::BoolValue(b)) => Ok(Val::Boolean(b)),
                Some(constant::ConstantKind::Int64Value(i)) => Ok(Val::Int(i)),
                Some(constant::ConstantKind::Uint64Value(u)) => Ok(Val::UInt(u)),
                Some(constant::ConstantKind::DoubleValue(d)) => Ok(Val::Double(d)),
                Some(constant::ConstantKind::StringValue(s)) => Ok(Val::String(s)),
                Some(constant::ConstantKind::BytesValue(b)) => Ok(Val::Bytes(b)),
                Some(constant::ConstantKind::DurationValue(_)) => {
                    Err(ConversionError::InvalidConstantKind)
                }
                Some(constant::ConstantKind::TimestampValue(_)) => {
                    Err(ConversionError::InvalidConstantKind)
                }
            }
        }
    }

    impl From<AstExpr> for expr::ExprKind {
        fn from(expr: AstExpr) -> Self {
            match expr {
                AstExpr::Ident(name) => expr::ExprKind::IdentExpr(expr::Ident { name }),
                AstExpr::Literal(val) => expr::ExprKind::ConstExpr(Constant::from(val)),
                _ => unimplemented!(),
            }
        }
    }

    impl TryFrom<expr::ExprKind> for AstExpr {
        type Error = ConversionError;

        fn try_from(expr_kind: expr::ExprKind) -> Result<Self, Self::Error> {
            match expr_kind {
                expr::ExprKind::IdentExpr(ident) => Ok(AstExpr::Ident(ident.name)),
                expr::ExprKind::ConstExpr(constant) => {
                    let val = Val::try_from(constant)?;
                    Ok(AstExpr::Literal(val))
                }
                _ => Err(ConversionError::InvalidExpressionKind),
            }
        }
    }

    impl From<IdedExpr> for types::Expr {
        fn from(ided_expr: IdedExpr) -> Self {
            types::Expr {
                id: ided_expr.id as i64,
                expr_kind: Some(expr::ExprKind::from(ided_expr.expr)),
            }
        }
    }

    impl TryFrom<types::Expr> for IdedExpr {
        type Error = ConversionError;

        fn try_from(expr: types::Expr) -> Result<Self, Self::Error> {
            let expr_kind = expr
                .expr_kind
                .ok_or_else(|| ConversionError::MissingField {
                    field: "expr_kind".to_string(),
                })?;

            let ast_expr = AstExpr::try_from(expr_kind)?;

            Ok(IdedExpr {
                id: expr.id as u64,
                expr: ast_expr,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_protobuf_types_accessible() {
            let _expr = types::Expr {
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

        #[test]
        fn test_bidirectional_conversion() {
            let test_cases = vec![
                // Basic cases
                Val::Null,
                Val::Boolean(true),
                Val::Boolean(false),
                Val::Int(42),
                Val::Int(-123),
                Val::UInt(42),
                Val::Double(3.14),
                Val::Double(-2.5),
                Val::String("hello world".to_string()),
                Val::String("".to_string()),
                Val::Bytes(vec![0x01, 0x02, 0x03]),
                Val::Bytes(vec![]),
                // Edge cases
                Val::Int(i64::MAX),
                Val::Int(i64::MIN),
                Val::UInt(u64::MAX),
                Val::Double(f64::INFINITY),
                Val::Double(f64::NEG_INFINITY),
                Val::Double(f64::NAN),
            ];

            for val in test_cases {
                let constant = Constant::from(val.clone());
                let converted_back = Val::try_from(constant).unwrap();

                // Handle NaN
                if let (Val::Double(orig), Val::Double(conv)) = (&val, &converted_back) {
                    if orig.is_nan() && conv.is_nan() {
                        continue;
                    }
                }

                assert_eq!(val, converted_back);
            }
        }

        #[test]
        fn test_constant_to_val_error_cases() {
            // Missing constant_kind
            let constant = Constant {
                constant_kind: None,
            };
            let result = Val::try_from(constant);
            assert_eq!(
                result,
                Err(ConversionError::MissingField {
                    field: "constant_kind".to_string()
                })
            );

            // Deprecated duration
            let constant = Constant {
                constant_kind: Some(constant::ConstantKind::DurationValue(
                    ::prost_types::Duration {
                        seconds: 60,
                        nanos: 0,
                    },
                )),
            };
            let result = Val::try_from(constant);
            assert_eq!(result, Err(ConversionError::InvalidConstantKind));

            // Deprecated timestamp
            let constant = Constant {
                constant_kind: Some(constant::ConstantKind::TimestampValue(
                    ::prost_types::Timestamp {
                        seconds: 1234567890,
                        nanos: 0,
                    },
                )),
            };
            let result = Val::try_from(constant);
            assert_eq!(result, Err(ConversionError::InvalidConstantKind));
        }

        #[test]
        fn test_expr_ident_conversion() {
            let ident_expr = AstExpr::Ident("variable_name".to_string());
            let expr_kind = expr::ExprKind::from(ident_expr.clone());

            if let expr::ExprKind::IdentExpr(ident) = expr_kind {
                assert_eq!(ident.name, "variable_name");
            } else {
                panic!("Expected IdentExpr");
            }

            let expr_kind = expr::ExprKind::IdentExpr(expr::Ident {
                name: "test_var".to_string(),
            });
            let converted_expr = AstExpr::try_from(expr_kind).unwrap();
            assert_eq!(converted_expr, AstExpr::Ident("test_var".to_string()));
        }

        #[test]
        fn test_expr_literal_conversion() {
            let literal_expr = AstExpr::Literal(Val::String("hello".to_string()));
            let expr_kind = expr::ExprKind::from(literal_expr.clone());

            if let expr::ExprKind::ConstExpr(constant) = expr_kind {
                assert_eq!(
                    constant.constant_kind,
                    Some(constant::ConstantKind::StringValue("hello".to_string()))
                );
            } else {
                panic!("Expected ConstExpr");
            }

            let expr_kind = expr::ExprKind::ConstExpr(Constant {
                constant_kind: Some(constant::ConstantKind::Int64Value(42)),
            });
            let converted_expr = AstExpr::try_from(expr_kind).unwrap();
            assert_eq!(converted_expr, AstExpr::Literal(Val::Int(42)));
        }

        #[test]
        fn test_ided_expr_conversion() {
            let ided_expr = IdedExpr {
                id: 123,
                expr: AstExpr::Ident("my_var".to_string()),
            };
            let protobuf_expr = types::Expr::from(ided_expr.clone());

            assert_eq!(protobuf_expr.id, 123);
            if let Some(expr::ExprKind::IdentExpr(ident)) = protobuf_expr.expr_kind {
                assert_eq!(ident.name, "my_var");
            } else {
                panic!("Expected IdentExpr");
            }

            let protobuf_expr = types::Expr {
                id: 456,
                expr_kind: Some(expr::ExprKind::IdentExpr(expr::Ident {
                    name: "another_var".to_string(),
                })),
            };
            let converted_ided = IdedExpr::try_from(protobuf_expr).unwrap();

            assert_eq!(converted_ided.id, 456);
            assert_eq!(
                converted_ided.expr,
                AstExpr::Ident("another_var".to_string())
            );
        }

        #[test]
        fn test_expr_conversion_errors() {
            let protobuf_expr = types::Expr {
                id: 1,
                expr_kind: None,
            };
            let result = IdedExpr::try_from(protobuf_expr);
            assert_eq!(
                result,
                Err(ConversionError::MissingField {
                    field: "expr_kind".to_string()
                })
            );

            let unsupported_expr_kind = expr::ExprKind::SelectExpr(Box::new(expr::Select {
                operand: Some(Box::new(types::Expr {
                    id: 1,
                    expr_kind: Some(expr::ExprKind::IdentExpr(expr::Ident {
                        name: "test".to_string(),
                    })),
                })),
                field: "field".to_string(),
                test_only: false,
            }));

            let result = AstExpr::try_from(unsupported_expr_kind);
            assert_eq!(result, Err(ConversionError::InvalidExpressionKind));
        }
    }
}
