#![feature(plugin)]
#![feature(box_syntax)]
#![feature(io)]

#![feature(plugin)]
#![plugin(phf_macros)]
extern crate phf;
extern crate byteorder;
extern crate libc;

#[cfg(not(test))]
use parser::Parser;
use ast::Ast;
use ast::dump::AstDumper;
use semck::SemCheck;

mod lexer;
mod error;
mod parser;
mod ast;
mod interner;
mod semck;
mod sym;


#[cfg(not(test))]
fn main() {
    match parse_file() {
        Ok(ast) => {
            AstDumper::new(&ast).dump();

            if let Err(errors) = SemCheck::new(&ast).check() {
                for err in &errors {
                    println!("{}", err);
                }

                println!("\n{} errors found.", errors.len());
            }
        }

        Err(err) => println!("{}", err)
    }
}

#[cfg(not(test))]
fn parse_file() -> Result<Ast, String> {
    let fname = try!(filename());
    let mut parser = try!(Parser::from_file(&fname).map_err(|_| { format!("can not read file {}", fname) }));
    let ast = try!(parser.parse().map_err(|e| e.message));

    Ok(ast)
}

#[cfg(not(test))]
fn filename() -> Result<String, String> {
    let mut args = std::env::args();

    args.nth(1).ok_or("file name expected".to_owned())
}
