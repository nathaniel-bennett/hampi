//! A simple utility to tokenize ASN files.

use std::io::{self, Write};

use clap::Parser;

use asn1_compiler::{
    generator::{Codec, Derive, Visibility},
    Asn1Compiler,
};

use std::fs::File;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(last = true)]
    files: Vec<String>,

    /// Name of the Rust Module to write generated code to.
    #[arg(short, long)]
    module: String,

    /// The name of the root ASN.1 structure to derive structured fuzzing routines for.
    #[arg(short, long, required = true)]
    root: String,

    #[arg(short, action=clap::ArgAction::Count)]
    debug: u8,

    /// Visibility of Generated Structures and members:
    #[arg(long, value_enum, default_value_t=Visibility::Public)]
    visibility: Visibility,

    /// ASN.1 Codecs to be Supported during code generation.
    /// Specify multiple times for multiple codecs. (eg. --codec aper --codec uper)
    #[arg(long, required = true)]
    codec: Vec<Codec>,

    /// Generate code for these derive macros during code generation.
    #[arg(long)]
    derive: Vec<Derive>,
}

fn main() -> io::Result<()> {
    let mut cli = Cli::parse();

    if cli.files.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No Input files Specified",
        ));
    }

    let derives = if cli.derive.contains(&Derive::All) {
        cli.derive
            .into_iter()
            .filter(|t| t == &Derive::All)
            .collect::<Vec<Derive>>()
    } else {
        if !cli.derive.contains(&Derive::Debug) {
            cli.derive.push(Derive::Debug);
        }
        cli.derive
    };

    let level = if cli.debug > 0 {
        if cli.debug == 1 {
            "debug"
        } else {
            "trace"
        }
    } else {
        "info"
    };

    let env = env_logger::Env::default().filter_or("MY_LOG_LEVEL", level);
    env_logger::init_from_env(env);

    let mut compiler = Asn1Compiler::new(
        &cli.module,
        &cli.visibility,
        cli.codec.clone(),
        derives.clone(),
    );
    compiler.compile_files(&cli.files)?;

    Ok(())
}

const PERFUZZ_H_CONTENTS: &'static [u8] = b"
#ifndef PERFUZZ_H
#define PERFUZZ_H

#ifdef __cplusplus
extern \"C\" {
#endif // __cplusplus

const long PERFUZZ_ERR_UNSPECIFIED = -1;

// Converts unstructured bytes into a structured PER message.
// Returns a the length of the structured bytes written to `buf_out`, or
// a negative error code on failure.
long perfuzz_structure(char *buf_in, long in_len, char *buf_out, long out_max);


#ifdef __cplusplus
}
#endif // __cplusplus

#endif // PERFUZZ_H
";

const PERFUZZ_LIB_CONTENTS: &'static [u8] = b"
#![allow(non_camel_case_types)]

mod {};

use std::os::raw::{c_char, c_uint};

";

fn compile_lib_files(module: &str, root: &str) {
    let mut output_file_h = File::create("perfuzz.h").unwrap();
    output_file_h.write_all(PERFUZZ_H_CONTENTS).unwrap();

    let mut output_file_lib = File::create("lib.rs").unwrap();

}
