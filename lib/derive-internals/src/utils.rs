use proc_macro2::{TokenStream, Ident};
use syn::{Item};
use std::io::{Write};
use std::fmt::Display;
use quote::quote;

pub fn is_derive_enabled(name: &str) -> bool {
    match std::env::var(format!("DEBUG_DERIVE_{}", name)) {
        Ok(s) => &*s == "1" || s.eq_ignore_ascii_case("true"),
        Err(_) => false,
    }
}

pub fn item_name(item: &Item) -> Box<dyn Display> {
    let ident: &Ident = match *item {
        Item::Fn(ref f) => &f.sig.ident,
        Item::Static(ref s) => &s.ident,
        Item::Struct(ref s) => &s.ident,
        _ => return Box::new(format!("{}", quote!(item))) as Box<dyn Display>
    };
    Box::new(ident.clone()) as Box<dyn Display>
}

pub fn debug_proc_macro(macro_name: &str, input: &dyn Display, result: &TokenStream) {
    if !is_derive_enabled(macro_name) { return }
    let original = format!("{}", result);
    match rustfmt_expr(&original) {
        Ok(formatted) => {
            eprintln!("{}!({}):", macro_name, input);
            for line in formatted.lines() {
                eprintln!("  {}", line);
            }
        },
        Err(error) => {
            eprintln!("{}!({}) caused rustfmt error:", macro_name, input);
            for line in error.lines() {
                eprintln!("  {}", line);
            }
            eprintln!("  original code: {}", original)
        }
    }
}

pub fn debug_derive(trait_name: &str, target: &dyn Display, result: &TokenStream) {
    if !is_derive_enabled(trait_name) { return }
    let original = format!("{}", result);
    match rustfmt(&original) {
        Ok(formatted) => {
            eprintln!("derive({}) for {}:", trait_name, target);
            for line in formatted.lines() {
                eprintln!("  {}", line);
            }
        },
        Err(error) => {
            eprintln!("derive({}) for {} caused rustfmt error:", trait_name, target);
            for line in error.lines() {
                eprintln!("  {}", line);
            }
            eprintln!("  original code: {}", original)
        }
    }
}

pub fn rustfmt_expr(target: &str) -> Result<String, String> {
    let dummy = format!(r#"fn expr() {{ {} }}"#, target);
    rustfmt(&dummy)
}

pub fn rustfmt(target: &str) -> Result<String, String> {
    use std::process::{Command, Stdio};
    let mut child = match Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
        Ok(child) => child,
        Err(_) => {
            /*
             * Assume we just couldn't find the command.
             * At this point invalid input couldn't have been
             * the cause of our error
             */
            return Ok(target.into())
        }
    };
    match child.stdin.as_mut().unwrap().write_all(target.as_bytes()).and_then(|()| {
        let output = child.wait_with_output()?;
        let utf_err = |cause: std::string::FromUtf8Error| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, cause)
        };
        let stdout = String::from_utf8(output.stdout)
            .map_err(utf_err)?;
        let stderr = String::from_utf8(output.stderr)
            .map_err(utf_err)?;
        Ok((output.status, stdout, stderr))
    }) {
        Ok((status, stdout, stderr)) => {
            if status.success() {
                Ok(stdout)
            } else {
                Err(stderr)
            }
        },
        Err(e) => {
            Err(format!("Unexpected IO error: {}", e))
        }
    }
}