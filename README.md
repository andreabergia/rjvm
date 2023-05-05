# RJVM

This project is an attempt to write a minimal JVM 7 using Rust.

Important note: **this is a hobby project, built for fun and for learning purposes**. In particular, it is my first real
program in Rust and I've used to learn the language - thus, I'm sure some parts of the code are not very "idiomatic"
Rust since I'm just learning the language.

The code quality is definitely not production ready - there are not enough tests, there isn't enough documentation and
some of the initial decision should be revisited.

The code is licensed under the [Apache v2 license](./LICENSE).

## What has been implemented

The current code can execute [various simple programs](./vm/tests/resources/rjvm), but it has a lot of limitations.

Things not implemented (and not planned to):

- generics
- threading
- multi dimensional arrays
- reflection
- annotations
- [class file verification](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.10)
- I/O

However, there's quite a few things implemented:

- parsing .class files
- class loading from a jar file or from a folder
- execution of code:
    - primitive types, arrays, strings
    - control flow statements
    - classes, subclasses, interfaces
    - methods (virtual, static)
    - exception throwing and catching
    - stack traces

The JVM uses the real classes from [OpenJDK 7](https://jdk.java.net/java-se-ri/7) - meaning the classes such as
`java.lang.Object`, `java.lang.String` or `java.lang.Exception` are _real_ classes, without any modifications. The JVM
is "good enough" to parse and execute their code.

## What has still to be implemented

Before declaring the project "complete", these are the things I still plan to implement:

- throwing real java exceptions (rather than internal errors that will abort executions) for things like stack overflow,
  accessing an array out of bounds, divisions by zero, etc
- review of the memory layout of objects
- garbage collection

There's also quite a few things whose implementation is quite poor, or not really coherent with the JVM specs,
but it is "good enough" to execute some simple code; for example arrays aren't real objects, or we don't really have the
concept of "identity hash code". However, it is unlikely I will fix those issues.

## Code structure

The code is currently structured in three crates:

- `utils`, which contains some common code, unrelated to the JVM;
    - this code is the oldest part of the project and is probably not particularly rust-idiomatic, or replaceable with
      some crates
- `reader`, which is able to read a `.class` file and contains various data structures for modelling their content;
- `vm`, which contains the virtual machine that can execute the code as a library
- `vm_cli`, which contains a very simple command-line launcher to run the vm, in the spirit of the `java` executable.

There are some unit test and some integration tests - probably not enough, but since this is not production code but
just a learning exercise, I'm not that worried about it.

I plan to extract `reader` class in a separate repository and publish it on [crates.io](https://crates.io/), since it
could actually be useful to someone else.
