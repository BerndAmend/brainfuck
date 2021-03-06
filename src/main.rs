use std::fs::File;
use std::io::prelude::*;

pub trait InputOutput {
    fn read(&mut self) -> Option<char>;
    fn write(&mut self, ch: char);
}

// Used for Benchmarks
pub struct DummyInputOutput;
impl InputOutput for DummyInputOutput {
    fn read(&mut self) -> Option<char> {
        None
    }
    fn write(&mut self, _: char) {}
}

// Used for tests
pub struct StringInputOutput {
    output: String,
}
impl StringInputOutput {
    fn new() -> StringInputOutput {
        StringInputOutput {
            output: String::new(),
        }
    }
}
impl InputOutput for StringInputOutput {
    fn read(&mut self) -> Option<char> {
        None
    }
    fn write(&mut self, ch: char) {
        self.output.push(ch);
    }
}

pub struct ConsoleInputOutput;
impl InputOutput for ConsoleInputOutput {
    fn read(&mut self) -> Option<char> {
        None
    }
    fn write(&mut self, ch: char) {
        print!("{}", ch);
    }
}

#[derive(Clone, Copy, Debug)]
enum Ops {
    Move(isize),
    Mod(i8),
    LoopOpen(usize),
    LoopClose(usize),
    SetCell(i8),
    SearchZeroCell(isize), // stores the step with
    Print,
    Read,
    End,
}

fn compile(source: &str) -> Result<Vec<Ops>, String> {
    let converted = source.chars().filter_map(|token| match token {
        '<' => Some(Ops::Move(-1)),
        '>' => Some(Ops::Move(1)),
        '-' => Some(Ops::Mod(-1)),
        '+' => Some(Ops::Mod(1)),
        '.' => Some(Ops::Print),
        ',' => Some(Ops::Read),
        '[' => Some(Ops::LoopOpen(0)),
        ']' => Some(Ops::LoopClose(0)),
        _ => None,
    });

    // Optimize
    let mut compiled = Vec::new();
    {
        let mut prepre = None;
        let mut pre = None;
        for cur in converted {
            match (prepre, pre, cur) {
                (_, Some(Ops::Move(v1)), Ops::Move(v2)) => {
                    pre = Some(Ops::Move(v1 + v2));
                }
                (_, Some(Ops::Mod(v1)), Ops::Mod(v2)) => {
                    pre = Some(Ops::Mod(v1 + v2));
                }
                (Some(Ops::LoopOpen(_)), Some(Ops::Mod(-1)), Ops::LoopClose(_)) => {
                    prepre = None;
                    pre = Some(Ops::SetCell(0));
                }
                (Some(Ops::LoopOpen(_)), Some(Ops::Move(n)), Ops::LoopClose(_)) => {
                    prepre = None;
                    pre = Some(Ops::SearchZeroCell(n));
                }
                (_, Some(Ops::SetCell(0)), Ops::Mod(v)) => {
                    pre = Some(Ops::SetCell(v));
                }
                _ => {
                    if let Some(o) = prepre {
                        compiled.push(o);
                    }
                    prepre = pre;
                    pre = Some(cur);
                }
            };
        }
        if let Some(o) = prepre {
            compiled.push(o);
        }
        if let Some(o) = pre {
            compiled.push(o);
        }
    }

    // calculate all loop jump destinations
    let mut stack: Vec<usize> = vec![];
    for i in 0..compiled.len() {
        match compiled[i] {
            Ops::LoopOpen(_) => stack.push(i),
            Ops::LoopClose(_) => {
                if let Some(start_pos) = stack.pop() {
                    compiled[start_pos] = Ops::LoopOpen(i);
                    compiled[i] = Ops::LoopClose(start_pos);
                } else {
                    return Err("missing [ for ]".into());
                }
            }
            _ => {
                // not relevant for this optimization
            }
        };
    }

    if stack.is_empty() {
        compiled.push(Ops::End);
        Ok(compiled)
    } else {
        Err("missing ] for [".into())
    }
}

fn execute(ops: &[Ops], in_out: &mut dyn InputOutput) {
    let mut memory = vec![0i8; 30000]; // a vec is much faster than a normal array
    let mut pos: usize = 0;
    let mut ip: usize = 0;

    'main: loop {
        match ops[ip] {
            Ops::Move(val) => pos = ((pos as isize) + val) as usize,
            Ops::Mod(val) => memory[pos] = memory[pos].wrapping_add(val),
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
            Ops::Print => in_out.write(memory[pos] as u8 as char),
            Ops::Read => memory[pos] = in_out.read().unwrap() as i8,
            Ops::End => break 'main,
        };
        ip += 1;
    }
}

pub fn run(filename: &str, in_out: &mut dyn InputOutput) {
    let mut file = File::open(filename).unwrap();
    let mut source =
        String::with_capacity(file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0));
    file.read_to_string(&mut source).unwrap();

    match compile(&source) {
        Ok(ops) => {
            // println!("{:?}", ops);
            execute(&ops, in_out)
        }
        Err(msg) => println!("Compilation error {}", msg),
    }
}

fn main() {
    let mut in_out = ConsoleInputOutput {};
    run(&std::env::args().nth(1).unwrap(), &mut in_out);
    println!("\nDone");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mandelbrot() {
        let filename = "programs/mandelbrot";
        let mut in_out = StringInputOutput::new();
        run(&format!("{}.bf", filename), &mut in_out);
        let mut f = File::open(&format!("{}.out", filename)).unwrap();
        let mut source = String::new();
        f.read_to_string(&mut source).unwrap();
        assert_eq!(source, in_out.output);
    }
}
