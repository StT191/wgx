
#[macro_export]
macro_rules! assoc {
    ($type:ty, $($field:ident: $value:expr),* ) => {
        {
            #[derive(Debug)]
            struct Assoc {
                $($field: $type),*
            }

            Assoc {
                $($field: $value.into()),*
            }
        }
    };
    ($($field:ident <$type:ty> : $value:expr),* ) => {
        {
            #[derive(Debug)]
            struct Assoc {
                $($field: $type),*
            }

            Assoc {
                $($field: $value.into()),*
            }
        }
    };
    ($key_type:ty => $value_type:ty, $($key:expr => $value:expr),+ ) => {
        {
            let mut hash_map: ::std::collections::HashMap<$key_type, $value_type>
                = ::std::collections::HashMap::new();
            $(hash_map.insert($key.into(), $value.into());)+
            hash_map
        }
    };
    ($($key:expr => $value:expr),+ ) => {
        {
            let mut hash_map = ::std::collections::HashMap::new();
            $(hash_map.insert($key, $value);)+
            hash_map
        }
    };
}
