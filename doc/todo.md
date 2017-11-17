# TODO

This is an assorted list of TODOs.

Some of these are just ideas, but are written down for posterity.

# All
- [ ] Flesh out service support and add implementation for each backend.
  * Service declaration have been simplified and are biased toward gRPC.
- [ ] Flesh out how breaking language changes are introduced.

# Compiler
- [x] ~~Markup for comments to use in documentation. Currently problematic since it would require two
      different parsing modes depending on context.~~
- [x] ~~Match statements using self type:~~
  * ~~Either: *reject* types referencing self, or externalize decoder for them.~~
- [x] Treat named types as (named) simple types, like tuples.
- [x] Enums.
- [x] Documentation syntax (preferably not comments).
- [x] finish arbitrary precision number representation.
- [ ] Validators
  * Tricky, too many extensions to DSL to be useful.
    Might just skip for now but push towards `2.0`.
- [ ] replace match value with instance.

# Repository
- [x] Simple filsystem prototype.
- [x] Support downloading specifications from a shared repository (over git + https).

# Maven Plugin + Maven Artifacts w/ Static Builds
- [x] Basic plugin for clean integration with Java ecosystem.
- [x] Version detection (like reproto-js).

# Java Backend
- [x] Generate _good_ builder.
- [x] Generated equals/hashCode (disabled when using lombok).
- [x] Generated toString (disabled when using lombok).
- [x] Tuple decoding (in `fasterxml`).
- [x] Support match statements through external deserializer model.

# Python Backend

- [x] ~~Declarations need to be hoisted to support static initialization of enums since they might
      depend on types in the same file. Alternatively, move initialization blocks to end of file.~~
- [x] Encode support (e.g. `instance.encode()`)
- [x] Relative import, especially with package prefixes. (not needed with aliases)
- [x] Create missing `__init__.py` files.
- [x] Array decoding.
- [x] Map decoding.
- [x] Tuple decoding.
- [x] Optional support.
- [x] Encode `type` field.
- [x] Enum support
- [ ] Strict decoding/encoding where types are deeply verified.

# DOC Backend
- [x] Generating documentation.
- [ ] Figure out how to do permanent links (similar to rustdoc?).
- [ ] Make better looking.

# JavaScript Backend
Necessary for browser compatibility.

- [x] Basic plugin.

# TypeScript Backend
Necessary for future browser compatibility.

- [ ] Basic plugin.
