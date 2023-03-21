# RJVM

This project is an attempt to write a minimal JVM using Rust.

It is a hobby project, built for fun and for learning purposes.

The code is licensed under the [Apache v2 license](./LICENSE).

## Status

The current code can execute a [very simple program](./vm/tests/resources/rjvm/SimpleMain.java).

- [ ] reading class files
  - [ ] class attributes
    - [ ] [InnerClasses](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.6)
    - [ ] [EnclosingMethod](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.7)
    - [ ] [synthetic](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.8)
    - [ ] [signature](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.9)
    - [ ] [SourceFile](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.10)
    - [ ] [SourceDebugExtension](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.11)
    - [ ] [deprecated](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.15)
    - [ ] [runtime visible annotations](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.16)
    - [ ] [runtime invisible annotations](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.17)
    - [ ] [BootstrapMethods](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.21)
  - [ ] methods
    - [ ] code
        - [ ] exception tables
        - [ ] attributes
          - [ ] [LineNumberTable](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.12)
          - [ ] [LocalVariableTable](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.13)
          - [ ] [LocalVariableTypeTable](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.14)
          - [ ] [StackMapTable](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.4)
    - [ ] source code mappings
    - [ ] attributes
      - [ ] [synthetic](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.8)
      - [ ] [signature](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.9)
      - [ ] [deprecated](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.15)
      - [ ] [exceptions](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.5)
      - [ ] [runtime visible annotations](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.16)
      - [ ] [runtime invisible annotations](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.17)
      - [ ] [runtime visible parameter annotations](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.18)
      - [ ] [runtime invisible parameter annotations](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.19)
      - [ ] [annotation default](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.20)
  - [ ] field
    - [ ] attributes
      - [x] constant value
      - [ ] [synthetic](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.8)
      - [ ] [signature](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.9)
      - [ ] [deprecated](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.15)
      - [ ] [runtime visible annotations](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.16)
      - [ ] [runtime invisible annotations](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.17)
- [ ] execution
  - [ ] data types
      - [ ] primitives
        - [x] int
        - [x] short
        - [x] char
        - [x] byte
        - [ ] long
        - [x] float
        - [ ] double
        - [x] boolean
        - [x] object
        - [ ] primitive arrays
        - [ ] object arrays
        - [ ] multidimensional arrays
  - [x] new object instances creation
  - [x] static methods invocation
  - [x] virtual methods invocation
  - [x] modelling of super classes
  - [x] abstract methods
  - [x] control flow
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
