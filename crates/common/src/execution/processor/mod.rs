use snafu::prelude::*;

use crate::compiler::{AddUntilZeroArg, Instruction, InstructionList};
use crate::execution::context::Context;
use crate::execution::memory::{Memory, MemoryError};

pub type Result<T> = std::result::Result<T, ProcessorError>;

struct Counter {
    val: usize,
}

impl Counter {
    fn new() -> Self {
        Self { val: 0 }
    }

    fn tick(&mut self) {
        self.val += 1;
    }

    fn jump(&mut self, target: usize) {
        self.val = target
    }

    fn get(&self) -> usize {
        self.val
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ProcessorState {
    Ready,
    Running,
    Halted,
    Failed,
}

pub struct Processor {
    counter: Counter,
    instructions: InstructionList,
    state: ProcessorState,
}

impl Processor {
    pub fn new(instructions: InstructionList) -> Self {
        Self {
            counter: Counter::new(),
            instructions,
            state: ProcessorState::Ready,
        }
    }

    fn abort(&mut self) {
        self.state = ProcessorState::Failed;
    }

    fn tick(&mut self) {
        self.counter.tick();
        self.check_halted();
    }

    fn check_halted(&mut self) {
        if self.instructions.0[self.counter.get()] == Instruction::Halt {
            self.state = ProcessorState::Halted;
        }
    }

    fn step(&mut self, context: &mut Context) -> Result<()> {
        let Context {
            memory,
            in_stream,
            out_stream,
        } = context;

        match self.state {
            ProcessorState::Halted => return Err(ProcessorError::AlreadyHalted),
            ProcessorState::Failed => return Err(ProcessorError::Failed),
            _ => {}
        }

        match &self.instructions.0[self.counter.get()] {
            Instruction::Add { val } => {
                if let Err(e) = memory.add(*val) {
                    self.abort();
                    Err(e.into())
                } else {
                    self.tick();
                    Ok(())
                }
            }
            Instruction::Seek { offset } => {
                if let Err(e) = memory.seek(*offset) {
                    self.abort();
                    Err(e.into())
                } else {
                    self.tick();
                    Ok(())
                }
            }
            Instruction::Clear => {
                memory.set(0).unwrap();
                self.tick();
                Ok(())
            }
            Instruction::AddUntilZero { target } => {
                if let Err(e) = self.add_while_zero(target, memory) {
                    self.abort();
                    Err(e)
                } else {
                    self.tick();
                    Ok(())
                }
            }
            Instruction::Input => {
                memory.set(in_stream.read()).unwrap();
                self.tick();
                Ok(())
            }
            Instruction::Output => {
                out_stream.write(memory.get());
                self.tick();
                Ok(())
            }
            Instruction::Jump { target } => {
                self.counter.jump(*target);
                self.check_halted();
                Ok(())
            }
            Instruction::JumpIfZero { target } => {
                if memory.get() == 0 {
                    self.counter.jump(*target);
                    self.check_halted();
                } else {
                    self.tick();
                }

                Ok(())
            }
            Instruction::Halt => {
                unreachable!()
            }
        }
    }

    fn add_while_zero(&self, target: &Vec<AddUntilZeroArg>, memory: &mut Memory) -> Result<()> {
        let val = memory.get();

        if val == 0 {
            return Ok(());
        }

        memory.set(0).unwrap();

        for AddUntilZeroArg { offset, times } in target {
            memory.seek(*offset)?;
            memory.add(val * *times)?;
            memory.seek(-*offset)?;
        }

        Ok(())
    }

    pub fn run(&mut self, context: &mut Context) -> Result<()> {
        match self.state {
            // There is only one halt instruction
            ProcessorState::Ready if self.instructions.0.len() == 1 => {
                return Err(ProcessorError::Empty)
            }
            ProcessorState::Halted => return Err(ProcessorError::AlreadyHalted),
            ProcessorState::Failed => return Err(ProcessorError::Failed),
            _ => {}
        }

        while self.state == ProcessorState::Ready || self.state == ProcessorState::Running {
            self.step(context)?
        }

        Ok(())
    }
}

#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum ProcessorError {
    #[snafu(display("invalid memory operation occurred"))]
    Memory { source: MemoryError },
    #[snafu(display("all instructions have already finished"))]
    AlreadyHalted,
    #[snafu(display("couldn't continue to run due to the previous error"))]
    Failed,
    #[snafu(display("empty program loaded"))]
    Empty,
}

impl From<MemoryError> for ProcessorError {
    fn from(e: MemoryError) -> Self {
        Self::Memory { source: e }
    }
}
