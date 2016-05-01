#![feature(test)]

extern crate test;

use std::io::prelude::*;
use std::fs::File;

pub trait InputOutput {
    fn read(&mut self) -> Option<char>;
    fn write(&mut self, ch: char);
}

pub struct DummyIntputOutput;
impl InputOutput for DummyIntputOutput {
    fn read(&mut self) -> Option<char> {
        None
    }
    fn write(&mut self, ch: char) {
    }
}

pub struct ConsoleIntputOutput;
impl InputOutput for ConsoleIntputOutput {
    fn read(&mut self) -> Option<char> {
        None
    }
    fn write(&mut self, ch: char) {
        print!("{}", ch);
    }
}

#[derive(Clone,Copy, Debug)]
enum Ops {
    Move(isize),
    Mod(i8),
    Print,
    Read,
    LoopOpen(usize),
    LoopClose(usize),
    SetCell(i8),
    SearchZeroCell(isize), // stores the stepwith
    End,
    NoOp,
}

fn compile(source: &str) -> Result<Vec<Ops>, String> {
    let source: String = source.chars()
                               .filter(|x| {
                                   match *x {
                                       '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']' => true,
                                       _ => false,
                                   }
                               })
                               .collect();
    let source = source.replace("[-]", "Z").replace("[<<<<<<<<<]", "L").replace("[>>>>>>>>>]", "R");
    let mut compiled = vec![];

    let mut next_op = Ops::NoOp;
    let mut previous = ' ';
    for token in source.chars() {

        let current_op = match token {
            '<' => Ops::Move(-1),
            '>' => Ops::Move(1),
            '-' => Ops::Mod(-1),
            '+' => Ops::Mod(1),
            '.' => Ops::Print,
            ',' => Ops::Read,
            '[' => Ops::LoopOpen(0),
            ']' => Ops::LoopClose(0),
            'Z' => Ops::SetCell(0),
            'L' => Ops::SearchZeroCell(-9),
            'R' => Ops::SearchZeroCell(9),
            _ => unreachable!(),
        };

        if previous == token {
            next_op = match (current_op, next_op) {
                (Ops::Move(v1), Ops::Move(v2)) => Ops::Move(v1 + v2),
                (Ops::Mod(v1), Ops::Mod(v2)) => Ops::Mod(v1 + v2),
                (Ops::SetCell(0), Ops::Mod(v)) => Ops::SetCell(v),
                _ => {
                    compiled.push(next_op);
                    current_op
                }
            }
        } else {
            compiled.push(next_op);
            next_op = current_op;
        }

        previous = token;
    }

    compiled.push(next_op);

    // find search zero cell commands

    // calculate all loop jump destinations
    let mut stack: Vec<usize> = vec![];
    for i in 0..compiled.len() {
        match compiled[i] {
            Ops::LoopOpen(_) => stack.push(i),
            Ops::LoopClose(_) => {
                match stack.pop() {
                    Some(start_pos) => {
                        compiled[start_pos] = Ops::LoopOpen(i);
                        compiled[i] = Ops::LoopClose(start_pos)
                    }
                    None => return Err("missing [ for ]".into()),
                }
            }
            _ => {
                // not relevant for this optimization
            }
        };
    }

    if stack.len() > 0 {
        Err("missing ] for [".into())
    } else {
        compiled.push(Ops::End);
        Ok(compiled)
    }
}

fn execute(ops: &Vec<Ops>, in_out: &mut InputOutput) {
    let ops = &ops[..];
    let mut memory = [0i8; 30000];
    let mut pos: usize = 0;
    let mut ip: usize = 0;

    'main: loop {
        match ops[ip] {
            Ops::Move(val) => pos = ((pos as isize) + val) as usize,
            Ops::Mod(val) => memory[pos] = memory[pos].wrapping_add(val),
            Ops::Print => in_out.write(memory[pos] as u8 as char),
            Ops::Read => memory[pos] = in_out.read().unwrap() as i8,
            Ops::LoopOpen(end) => {
                if memory[pos] == 0 {
                    ip = end;
                }
            }
            Ops::LoopClose(start) => {
                if memory[pos] != 0 {
                    ip = start;
                }
            }
            Ops::SetCell(value) => memory[pos] = value,
            Ops::SearchZeroCell(step) => {
                while memory[pos] != 0 {
                    pos = ((pos as isize) + step) as usize;
                }
            }
            Ops::End => break 'main,
            Ops::NoOp => {}
        };
        ip += 1;
    }
}

pub fn run(filename: &str, in_out: &mut InputOutput) {
    let mut f = File::open(filename).unwrap();
    let mut source = String::new();
    f.read_to_string(&mut source).unwrap();

    match compile(&source) {
        Ok(ops) => {
            // println!("{:?}", ops);
            execute(&ops, in_out)
        }
        Err(msg) => println!("Compilation error {}", msg),
    }
}

fn main() {
    let mut in_out = ConsoleIntputOutput {};
    run(&std::env::args().nth(1).unwrap(), &mut in_out);
    println!("\nDone");
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

     #[bench]
    fn mandelbrot(b: &mut Bencher) {
        b.iter(|| {
            let mut in_out = DummyIntputOutput {};
            run("programs/mandelbrot.bf", &mut in_out);
        });
    }
}
