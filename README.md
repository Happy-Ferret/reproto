# ReProto
[![Build Status](https://travis-ci.org/reproto/reproto.svg?branch=master)](https://travis-ci.org/reproto/reproto)
[![crates.io](https://img.shields.io/crates/v/reproto.svg)](https://crates.io/crates/reproto)

ReProto is a system for describing and handling dependencies for JSON schemas.

It has the following components:

* An interface description language (IDL) called reproto.
* A compiler for the reproto language.
* A [semantic version checker][semck] which verifies that modifications to schemas do not violate
  [semantic versioning].
* A build system and package manager similar to [Cargo].
  This handles downloading, building, and publishing of dependencies.

For more information:

* See [Specification][spec] for details on how the reproto language works.
* See [TODO][todo] for a list of things that still need to be done.
* See [Examples][examples] for some example protocol specifications.
* See [Config][config] for how to configure ReProto.
* See [Integration Tests][it] for some examples of how protocol specifications can be used.

**Note:** This project is in an early stage. Things will change a lot. Please take it for a spin,
but avoid building large repositories of specifications right now.

# Backends

* Java (`java`)
  * Data models using [fasterxml jackson][jackson] (`-m fasterxml`), and/or
    [lombok][lombok] (`-m lombok`).
  * [gRPC][grpc] support through the `grpc` module.
* JavaScript (`js`)
  * ES2015 classes, that can be transpiled using babel (see [Integration Test][js-it]).
* Python (`python`)
  * Plain-python classes, compatible with 2 and 3 for databinding.
* Rust (`rust`)
  * Serde-based serialization.
* Doc (`doc`)
  * HTML-based documentation, based from contextual markdown comments.

# Examples

Make you have [gotten started with Rust][rust-get-started].

Build ReProto using cargo:

```bash
$> cargo install --path $PWD/cli reproto
```

This will install `reproto` into `~/.cargo/bin`, make sure it is in your PATH.

The following is an example of how to build documentation for a the [examples manifest][examples].

```bash
$> cd examples
$> reproto doc -o target/doc
$> open target/doc/index.html
```

For more examples, please have a look at our [integration tests][it].

[examples]: /examples/reproto.toml
[rust-get-started]: https://doc.rust-lang.org/book/getting-started.html
[it]: /it

## [Maven Plugin][maven-plugin]

A Maven plugin that integrates reproto into the build lifecycle of a maven project.

[maven-plugin]: https://github.com/reproto/reproto-maven-plugin

## [VIM Plugin][vim]

A VIM plugin that provides syntax highlighting.

[vim]: https://github.com/reproto/reproto-vim

# Testing

This project includes an extensive set of integration tests.

See `make help` for documentation on what can be done.

Suites are tests which compiled a given set of rules, and compares with expected output.

Projects are complete project tests.
These are projects written for various programming languages, and are generally harder to build.

The tool [`check-project-deps`](tools/check-project-deps) is used to determine
which projects your local system can build.

To run all tests, do:

```bash
$> make clean all
```

# The IDL

ReProto is an interface description language (IDL) for schemas.

The schema describe the structure of JSON documents, which is necessary to generate data structures
in variour programming languages for safely and convenient interaction.

The goal is to have a compact, intuitive, and productive language for writing specifications.

You can find example specifications under the [examples] directory.

[Cargo]: https://github.com/rust-lang/cargo
[config]: /doc/config.md
[examples]: /examples
[grpc]: https://grpc.io
[idl]: #the-idl
[it]: /it
[jackson]: https://github.com/FasterXML/jackson-databind
[js-it]: /it/js
[lombok]: https://projectlombok.org/
[semantic versioning]: https://semver.org
[semck]: /semck
[spec]: /doc/spec.md
[todo]: /doc/todo.md
