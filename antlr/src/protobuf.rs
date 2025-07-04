#[cfg(feature = "protobuf")]
pub mod protobuf {
    pub mod types {
        include!(concat!(env!("OUT_DIR"), "/cel.expr.rs"));
    }

    pub use types::*;

    use crate::reference::Val;

    #[derive(Debug, Clone, PartialEq)]
    pub enum ConversionError {
        MissingField { field: String },
        InvalidConstantKind,
    }

    impl std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ConversionError::MissingField { field } => {
                    write!(f, "Missing required field: {}", field)
                }
                ConversionError::InvalidConstantKind => {
                    write!(f, "Invalid constant kind")
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
    }
}
