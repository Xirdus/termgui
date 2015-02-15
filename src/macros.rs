#![macro_use]

macro_rules! trivial_error {
    ($($name:ident = $desc:expr;)+) => (
        use std::error::Error;
        use std::fmt::Display;
        use std::fmt::Formatter;
        use std::fmt::Error as FmtError;
        $(
            const DESC: &'static str = $desc;
            pub struct $name;

            impl Display for $name {
                fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
                    DESC.fmt(f)
                }
            }

            impl Error for $name {
                fn description(&self) -> &str {
                    DESC
                }

                fn cause(&self) -> Option<&Error> {
                    None
                }
            }
        )+
    )
}
