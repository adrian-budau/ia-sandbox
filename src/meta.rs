macro_rules! impl_child_error {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $(#[$meta])*
        pub enum $name {
            $($body)*
        }

        use std::fmt;
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                impl_child_error! {
                    match self {
                        @variants_display f $name [$($body)*],
                    }
                }
            }
        }

        use std::error;
        impl error::Error for $name {
            fn description(&self) -> &str {
                impl_child_error! {
                    match self {
                        @variants_error $name [$($body)*],
                    }
                }
            }

            fn cause(&self) -> Option<&error::Error> {
                None
            }
        }
    };

    /* End impl Error */
    (
        match $self:ident {
            @variants_error $name:ident [],
            $($body:tt)*
        }
    ) => {
        match $self {
            $($body)*
        }
    };

    /* Tuple variant impl Error */
    (
        match $self:ident {
            @variants_error $name:ident [ $variant:ident($arg:ty), $($tail:tt)* ],
            $($body:tt)*
        }
    ) => {
        impl_child_error! {
            match $self {
                @variants_error $name [ $($tail)* ],
                &$name::$variant(_) => stringify!($variant),
                $($body)*
            }
        }
    };

    /* Struct variant for impl Error */
    (
        match $self:ident {
            @variants_error $name:ident [ $variant:ident {
                $($arg:ident: $type:ty,)*
            }, $($tail:tt)*],
            $($body:tt)*
        }
    ) => {
        impl_child_error! {
            match $self {
                @variants_error $name [$($tail)*],
                &$name::$variant { .. } => stringify!($variant),
                $($body)*
            }
        }
    };

    /* End impl Display */
    (
        match $self:ident {
            @variants_display $f:ident $name:ident [],
            $($body:tt)*
        }
    ) => {
        match $self {
            $($body)*
        }
    };

    /* Tuple variant for impl Display */
    (
        match $self:ident {
            @variants_display $f:ident $name:ident [ $variant:ident($arg:ty), $($tail:tt)* ],
            $($body:tt)*
        }
    ) => {
        impl_child_error! {
            match $self {
                @variants_display $f $name [ $($tail)* ],
                &$name::$variant(ref arg) => {
                    write!($f, concat!(stringify!($variant), ": {}"), arg)
                },
                $($body)*
            }
        }
    };

    /* Struct variant for impl Display */
    (
        match $self:ident {
            @variants_display $f:ident $name:ident [ $variant:ident {
                $($arg:ident: $type:ty,)*
            }, $($tail:tt)*],
            $($body:tt)*
        }
    ) => {
        impl_child_error! {
            match $self {
                @variants_display $f $name [ $($tail)* ],
                &$name::$variant {
                    $(ref $arg,)*
                } => {
                    write!($f, concat!(stringify!($variant), " with "))?;
                    $(
                        write!($f, concat!(stringify!($arg), ": {}"), $arg)?;
                    )*
                    write!($f, "")
                },
                $($body)*
            }
        }
    }
}
