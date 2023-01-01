use std::fmt::Write;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum PrintOutput {
    Null,
    StdOut,
    StdErr,
    Str(String),
}

impl Write for PrintOutput {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        match self {
            PrintOutput::Null => (),
            PrintOutput::StdOut => print!("{s}"),
            PrintOutput::StdErr => eprint!("{s}"),
            PrintOutput::Str(string) => string.push_str(s),
        }
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Config {
    pub vm_debug: PrintOutput,
    pub vm_error: PrintOutput,
    pub compiler_debug: PrintOutput,
    pub compiler_error: PrintOutput,
    pub print_output: PrintOutput,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vm_debug: PrintOutput::Null,
            vm_error: PrintOutput::StdErr,
            compiler_debug: PrintOutput::Null,
            compiler_error: PrintOutput::StdErr,
            print_output: PrintOutput::StdOut,
        }
    }
}
