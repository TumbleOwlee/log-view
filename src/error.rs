
pub enum Error {
   NoPipeIn,
   FileOpenFailed(String),
   CustomThemeFailed(String)
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoPipeIn => f.write_fmt(format_args!("Failed to initialze input pipe. Did you forget to pipe something?")),
            Error::FileOpenFailed(s) => f.write_fmt(format_args!("Failed to initialze input file '{}'. Does the file exist?", s)),
            Error::CustomThemeFailed(s) => f.write_fmt(format_args!("Couldn't load custom theme from '{}'.", s))
        }
    }
}
