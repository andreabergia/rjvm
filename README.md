# RJVM

This project is an attempt to write a minimal JVM 7 using Rust.

Important note: **this is a hobby project, built for fun and for learning purposes**. In particular, it is my first real
program in Rust and I've used to learn the language - thus, I'm sure some parts of the code are not very "idiomatic"
Rust since I'm just learning the language.

The code quality is definitely not production ready - there are not enough tests, there isn't enough documentation and
some of the initial decision should be revisited. (I.e.: this is not representative of the code I write for work 😊.)

The code is licensed under the [Apache v2 license](./LICENSE).

The architecture is discussed in a series of posts on my blog, [https://andreabergia.com](https://andreabergia.com/series/writing-a-jvm-in-rust/).

## What has been implemented and what hasn't

The current code can execute [various simple programs](./vm/tests/resources/rjvm), but it has a lot of limitations.

Here is a list of the implemented features:

- parsing .class files
- resolving classes from a jar file, or from a folder
- execution of real code:
    - primitive types, arrays, strings
    - control flow statements
    - classes, subclasses, interfaces
    - methods (virtual, static, natives)
    - exception throwing and catching
    - stack traces
    - garbage collection

However, there are a lot of important things not implemented (and not planned to):

- threading
- multi dimensional arrays
- reflection
- annotations
- [class file verification](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.10)
- I/O
- just in time code execution (JIT)
- proper class loaders

The JVM uses the _real classes_ from [OpenJDK 7](https://jdk.java.net/java-se-ri/7) - meaning the classes such as
`java.lang.Object`, `java.lang.String` or `java.lang.Exception` are real production classes, without any modifications.
The JVM is "good enough" to parse and execute their code, something which makes me very happy indeed. 😊

The VM is limited to 64 bits platforms, as there are quite a few places where I assume that the size of a pointer
is exactly 8 bytes.

## Implementations that should be modified

One poor implementation detail is that for things like stack overflow, accessing an array out of bounds, divisions by
zero, etc. I should be throwing real java exceptions, rather than internal errors that will abort executions.
In general, the error handling is not great - there are no details when you get an internal error, something that made
debugging more painful than it should have been.

There's also quite a few things whose implementation is quite poor, or not really coherent with the JVM specs,
but it is "good enough" to execute some simple code; for example I do not have a class for arrays. If you're curious,
look for the TODO in the code.

I'm also quite sure there's a million bugs in the code. 😅

## Code structure

The code is currently structured in three crates:

- `reader`, which is able to read a `.class` file and contains various data structures for modelling their content;
- `vm`, which contains the virtual machine that can execute the code as a library;
- `vm_cli`, which contains a very simple command-line launcher to run the vm, in the spirit of the `java` executable.

There are some unit test and some integration tests - definitely not enough, but since this is not production code but
just a learning exercise, I'm not that worried about it. Still, IntelliJ tells me I have a bit above 80% of coverage,
which is not bad. The error paths aren't really tested, though.

I use [just](https://github.com/casey/just) as a command runner, but most tasks are just cargo commands.

# Project status and further works

I consider the project complete. It was super instructive, but I do not plan to keep working on it.

The only thing I'm considering is to extract the `reader` crate in a separate repository, and publish it on
[crates.io](https://crates.io/), since it could actually be useful to someone else.
