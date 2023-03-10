# RJVM

This project is an attempt to write a minimal JVM using Rust.

It is a hobby project, built for fun and for learning purposes.

The code is licensed under the [Apache v2 license](./LICENSE).

## Status

The current code can execute a [very simple program](./vm/tests/resources/rjvm/SimpleMain.java).

- [ ] reading class files
  - [ ] source code mappings
  - [ ] exception tables
  - [ ] annotations
  - [ ] other code attributes
- [ ] execution
  - [ ] data types
      - [ ] primitives
        - [x] int
        - [ ] short
        - [ ] char
        - [ ] byte
        - [ ] long
        - [ ] float
        - [ ] double
        - [ ] boolean
        - [x] object
        - [ ] arrays
  - [x] new object instances creation
  - [x] static methods invocation
  - [x] virtual methods invocation
  - [x] modelling of super classes
  - [ ] object allocations and garbage collection
  - [ ] exceptions
  - [ ] threading
  - [ ] tons of other features :-)

## Structure

The code is currently structured in three crates:

- `utils`, which contains some common code, unrelated to the JVM;
- `reader`, with the `.class` file reader and the data structure for modelling them;
- `vm`, which contains the virtual machine that can execute the code.

There are some unit test and some integration tests - probably not enough, 
but since this is not production code but just a learning exercise,
I'm not that worried about it.
