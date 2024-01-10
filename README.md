# Assoc Type Params

Experiment using single type with associated types to track type params.

Instead of:

```rust
struct CmdCtx<Input, Output, Error> {
    input: Input,
    output: Output,
    marker: std::marker::PhantomData<Error>,
}

fn borrows_cmd_ctx<Input, Output, Error>(
    ctx: &mut CmdCtx<Input, Output, Error>,
    // other params
) -> Result<(), Error> {
    todo!()
}

fn pass_cmd_ctx_around<Input, Output, Error>(
    cmd_ctx: &mut CmdCtx<Input, Output, Error>,
) {
    todo!()
}
```

We want something like:

```rust
struct CmdCtx<Types> {
    input: Types::Input,
    output: Types::Output,
}

fn borrows_cmd_ctx<Types>(
    ctx: &mut CmdCtx<Types>,
    // other params
) -> Result<(), Types::Error> {
    todo!()
}

fn pass_cmd_ctx_around<Types>(
    cmd_ctx: &mut CmdCtx<Types>,
) {
    todo!()
}
```
