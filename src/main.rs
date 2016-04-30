use std::io::prelude::*;
use std::fs::File;

#[derive(Clone)]
enum Ops {
    Right,
    Left,
    Incr,
    Decr,
    Print,
    LoopOpen{end: usize},
    LoopClose{start: usize},
    End,
}

fn compile(source: &str) -> Result<Vec<Ops>, String> {
    let source = source.chars().filter(|x| match *x {
        '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']' => true,
        _ => false
        }).collect::<Vec<_>>();
    let mut compiled = vec![];
    compiled.reserve(source.len()+1);

    let mut stack: Vec<usize> = vec![];

    for i in 0..source.len() {
        let new_op = match source[i] {
            '>'	=> Ops::Right,
            '<'	=> Ops::Left,
            '+'	=> Ops::Incr,
            '-'	=> Ops::Decr,
            '.'	=> Ops::Print,
            ','	=> return Err(", is not implemented".into()),
            '['	=> {
                    stack.push(i);
                    Ops::LoopOpen{end: 0}
                    },
            ']'	=> match stack.pop() {
                        Some(start_pos) => {
                            compiled[start_pos] = Ops::LoopOpen{ end: i };
                            Ops::LoopClose{start: start_pos}
                        },
                        None => return Err("missing [ for ]".into()),
                    },
            _ => unreachable!(),
        };
        compiled.push(new_op);
    }

    if stack.len() > 0 {
        Err("missing ] for [".into())
    } else {
        compiled.push(Ops::End);
        Ok(compiled)
    }
}

fn execute(ops: &Vec<Ops>) {
    let ops = &ops[..];
    let mut memory = [0i8; 30000];
    let mut pos: usize = 0;
    let mut ip: usize = 0;

    'main: loop {
        match ops[ip] {
            Ops::Right => pos += 1,
            Ops::Left => pos -= 1,
            Ops::Incr => memory[pos] = memory[pos].wrapping_add(1),
            Ops::Decr => memory[pos] = memory[pos].wrapping_sub(1),
            Ops::Print => print!("{}", memory[pos] as u8 as char),
            Ops::LoopOpen{end} => if memory[pos] == 0 { ip = end; },
            Ops::LoopClose{start} => if memory[pos] != 0 { ip = start; },
            Ops::End => break 'main,
        };
        ip += 1;
    }
}

fn run(source: &str) {
    match compile(source) {
        Ok(ops) => execute(&ops),
        Err(msg) => println!("Compilation error {}", msg),
    }
}

fn main() {
    let mut f = File::open(std::env::args().nth(1).unwrap()).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    run(&s);
    println!("\nDone");
}
