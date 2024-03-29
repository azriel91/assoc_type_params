use std::{
    fmt::{self, Display},
    io::{Stdin, Stdout, Write},
};

// === Traits for pluggable types, with compile time safety / static checking.
// === //

trait Input {
    fn read(&mut self) -> Result<String, FrameworkError>;
}

trait Output {
    fn write(&mut self, s: &str) -> Result<(), FrameworkError>;
}

trait Logic {
    type ReturnType;
    type Error: std::error::Error;

    fn do_work(&mut self) -> Result<Self::ReturnType, Self::Error>;
}

/// Trait that tracks all associated types;
trait TypeParamsT {
    type AppError;
    type Input;
    type Output;
}

/// Similar to `TypeParamsT`, with bounds added for compile time safety.
///
/// The associated types of `TypeParamsT` must be specified, otherwise we
/// haven't told Rust that the supertrait's associated type must be exactly the
/// same as this trait's associated types.
///
/// Much thanks to `@quinedot`.
///
/// See <https://users.rust-lang.org/t/trait-bounds-transitive-inference/105118>
trait TypeParamsConstrained:
    TypeParamsT<
        AppError = <Self as TypeParamsConstrained>::AppError,
        Input = <Self as TypeParamsConstrained>::Input,
        Output = <Self as TypeParamsConstrained>::Output,
    >
{
    type AppError: std::error::Error + 'static;
    type Input: Input + 'static;
    type Output: Output + 'static;
}

impl<T> TypeParamsConstrained for T
where
    T: TypeParamsT,
    T::AppError: std::error::Error + 'static,
    T::Input: Input + 'static,
    T::Output: Output + 'static,
{
    type AppError = T::AppError;
    type Input = T::Input;
    type Output = T::Output;
}

// === Error / Value types === //

#[derive(Debug)]
struct LogicError(String);

impl std::error::Error for LogicError {}

impl Display for LogicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
enum FrameworkError {
    Logic(LogicError),
    Input(std::io::Error),
    Output(std::io::Error),
}

impl From<LogicError> for FrameworkError {
    fn from(error: LogicError) -> Self {
        Self::Logic(error)
    }
}

impl std::error::Error for FrameworkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameworkError::Logic(error) => Some(error),
            FrameworkError::Input(error) => Some(error),
            FrameworkError::Output(error) => Some(error),
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl Display for FrameworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameworkError::Logic(_) => write!(f, "Logic error"),
            FrameworkError::Input(_) => write!(f, "Input error"),
            FrameworkError::Output(_) => write!(f, "Output error"),
        }
    }
}

// === Context capturing types === //

struct CmdCtx<Types>
where
    Types: TypeParamsT,
{
    input: Types::Input,
    output: Types::Output,
}

// === Concrete implementations of pluggable types === //

impl Input for Stdin {
    fn read(&mut self) -> Result<String, FrameworkError> {
        let mut buffer = String::with_capacity(256);
        let _n = self.read_line(&mut buffer).map_err(FrameworkError::Input)?;

        Ok(buffer)
    }
}

impl Output for Stdout {
    fn write(&mut self, s: &str) -> Result<(), FrameworkError> {
        self.lock()
            .write_all(s.as_bytes())
            .map_err(FrameworkError::Output)
    }
}

struct StdioEndpoint;
impl TypeParamsT for StdioEndpoint {
    type AppError = FrameworkError;
    type Input = Stdin;
    type Output = Stdout;
}

struct WorkLogic;
impl Logic for WorkLogic {
    type Error = LogicError;
    type ReturnType = u8;

    fn do_work(&mut self) -> Result<Self::ReturnType, Self::Error> {
        Ok(123)
    }
}

// === User level logic === //

fn run<Types, L>(
    cmd_ctx: &mut CmdCtx<Types>,
    logic: &mut L,
) -> Result<L::ReturnType, <Types as TypeParamsConstrained>::AppError>
where
    Types: TypeParamsConstrained,
    L: Logic,
    <Types as TypeParamsConstrained>::AppError: From<L::Error> + From<FrameworkError>,
    //
    // These bounds don't have to be specified individually, since Rust can infer them from
    // `TypeParamsConstrained`.
    //
    // <Types as TypeParamsT>::Output: Output,
    // <Types as TypeParamsT>::Input: Input,
{
    let CmdCtx { input, output } = cmd_ctx;

    output.write("Enter some input:\n")?;

    let line = input.read()?;
    let t = logic.do_work()?;

    output.write("You entered: ")?;
    output.write(&line)?;

    Ok(t)
}

// TODO: CmdCtxBuilder, slowly building up type tracker?.

fn main() -> Result<(), FrameworkError> {
    let mut cmd_ctx = CmdCtx::<StdioEndpoint> {
        input: std::io::stdin(),
        output: std::io::stdout(),
    };

    let value = run(&mut cmd_ctx, &mut WorkLogic)?;
    println!("Return value: {value}.");

    Ok(())
}
