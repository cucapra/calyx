/// Define a primitive
///
/// ```
/// primitive!(StdAdd[width](left: width, right: width) {
///   let left_64 = left.as_u64();
///   let right_64 = right.as_u64();
///   let init_val = left_64 + right_64;
///   let bitwidth: usize = left.vec.len();
///   Value::from_init(init_val, bitwidth).into()
/// });
/// ```
/// The macro implementes the [[Primitive]] trait for the struct.
///
/// TODO(rachit): $out_width is never used.
#[macro_export]
macro_rules! comb_primitive {
    ($name:ident[
        $( $param:ident ),+
    ]( $( $port:ident : $width:ident ),+ ) ->
     ( $( $out:ident : $out_width:ident ),+ ) $execute:block
    ) => {
        #[derive(Clone, Debug, Default)]
        #[allow(non_snake_case)]
        pub struct $name {
            $($param: u64),+
        }

        impl $name {
            pub fn new(params: calyx::ir::Binding) -> Self {
                let mut base = Self::default();
                for (param, value) in params {
                    match param.as_ref() {
                        $( $crate::in_fix!($param) => base.$param = value ),+,
                        p => unreachable!(format!("Unknown parameter: {}", p)),
                    }
                }
                base
            }

            #[allow(non_snake_case)]
            pub fn from_constants($( $param: u64 ),+) -> Self {
                $name {
                    $($param),+
                }
            }
        }


        impl Primitive for $name {

            fn is_comb(&self) -> bool { true }

            fn validate(
                &self,
                inputs: &[(calyx::ir::Id, &crate::values::Value)]
            ) {
                for (id, v) in inputs {
                    match id.as_ref() {
                        $( $crate::in_fix!($port) => assert_eq!(v.len() as u64, self.$width) ),+,
                        p => unreachable!(format!("Unknown port: {}", p)),
                    }
                }
            }

            #[allow(non_snake_case,unused)]
            fn execute(
                &mut self,
                inputs: &[(calyx::ir::Id, &crate::values::Value)],
                // done_val not used in combinational primitives
                _done_val: Option<&crate::values::Value>
            ) -> Vec<(calyx::ir::Id, crate::values::OutputValue)> {

                #[derive(Default)]
                struct Ports<'a> {
                    $( $port: Option<&'a crate::values::Value> ),+
                }

                let mut base = Ports::default();

                for (id, v) in inputs {
                    match id.as_ref() {
                        $( $crate::in_fix!($port) => base.$port = Some(v) ),+,
                        p => unreachable!(format!("Unknown port: {}", p)),
                    }
                }

                let exec_func = |$($param: u64),+, $( $port: &Value ),+| -> crate::values::OutputValue {
                    $execute
                };

                #[allow(unused_parens)]
                let ($( $out ),+) = exec_func(
                    $(self.$param),+,
                    $( base
                        .$port
                        .expect(&format!("No value for port: {}", $crate::in_fix!($port)).to_string()) ),+
                );

                return vec![
                    $( ($crate::in_fix!($out).into(), $out) ),+
                ]

            }

            // Combination components cannot be reset
            fn reset(
                &mut self,
                inputs: &[(calyx::ir::Id, &crate::values::Value)],
            ) -> Vec<(calyx::ir::Id, crate::values::OutputValue)> {
                self.execute(inputs, /* Value is not used */ None)
            }

            // No-op for combinational primitives.
            fn commit_updates(&mut self) {}

            // No-op for combinational primitives.
            fn clear_update_buffer(&mut self) {}
        }
    };
}
#[macro_export]
macro_rules! in_fix {
    ( r#in ) => {
        stringify!(in)
    };
    ( $name:ident ) => {
        stringify!($name)
    };
}
