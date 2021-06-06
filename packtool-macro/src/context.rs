use std::{fmt, mem, thread};

use quote::ToTokens;

#[derive(Default)]
pub struct Context {
    errors: Vec<syn::Error>,
    checked: bool,
}

impl Context {
    pub fn add_error(&mut self, error: syn::Error) {
        self.errors.push(error);
    }

    pub fn add_error_by<T, M>(&mut self, tokens: T, message: M)
    where
        T: ToTokens,
        M: fmt::Display,
    {
        self.add_error(syn::Error::new_spanned(tokens.into_token_stream(), message))
    }

    pub fn check(mut self) -> Result<(), Vec<syn::Error>> {
        self.checked = true;
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(mem::replace(&mut self.errors, Vec::new()))
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if !thread::panicking() && !self.checked {
            panic!("Errors were reported but they have not been checked")
        }
    }
}
