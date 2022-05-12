# Store

A Nix store is a collection of [store objects](objects.md) which refer to one another.

These store objects can hold arbitrary data, and Nix makes no distinction if they are used as build inputs, build results, or build tasks.

A Nix store allows adding, retrieving, and deleting store objects.
It can perform builds, that is, transform build inputs using instructions from the build tasks into build outputs.
It also keeps track of *references* between data and can therefore garbage-collect unused store objects.

## Two models, abstract and concrete

The Nix store layer is the heart of Nix, the cornerstone of its design.
It comes from two basic insights: a vision for build systems in the abstract based on functional programming, and an application of the vision to conventional software for conventional operating system.
We could just present the combination of those two in the form of the current design of the Nix store, but we believe there is value introducing them separately.
This still describes how Nix works, so this section still serves as a spec, but it also demonstrates with Nix's authors believe is a good way to think* about Nix.
If one tries to learn the concrete specifics before learning the abstract model, the following text might come across as a wall of details without sufficient motivation.
Conversely, if one learns the abstract model first, many of the concrete specifics will make more sense as miscellaneous details placed in the "slots" where the abstract model expects.
The hope is that makes the material far less daunting, and helps it make sense in the mind of the reader.

## A [Rosetta stone](https://en.m.wikipedia.org/wiki/Rosetta_Stone) for build system terminology

Nix is far from the other project to try to envision build systems abstractly, and indeed the design of the Nix store is comparable to other work.
Usage of terms is, for historic reasons, not entirely consistent within the Nix ecosystem, and still subject to slow change.

The following translation table points out similarities and equivalent terms, to help clarify their meaning and inform consistent use in the future.

generic build system | Nix | [Bazel](https://bazel.build/start/bazel-intro) | [Build Systems à la Carte](https://www.microsoft.com/en-us/research/uploads/prod/2018/03/build-systems.pdf) | programming language
-- | -- | -- | -- | --
data (build input, build result) | store object | [artifact](https://bazel.build/reference/glossary#artifact) | value | value
build instructions | builder | ([depends on action type](https://docs.bazel.build/versions/main/skylark/lib/actions.html)) | function | function
build task | derivation | [action](https://bazel.build/reference/glossary#action) | `Task` | [thunk](https://en.m.wikipedia.org/wiki/Thunk)
build plan | derivation graph | [action graph](https://bazel.build/reference/glossary#action-graph), [build graph](https://bazel.build/reference/glossary#build-graph) | `Tasks` | [call graph](https://en.m.wikipedia.org/wiki/Call_graph)
build | build | build | application of `Build` | evaluation
persistence layer | store | [action cache](https://bazel.build/reference/glossary#action-cache) | `Store` | heap

All of these systems share features of [declarative programming](https://en.m.wikipedia.org/wiki/Declarative_programming) languages, a key insight first put forward by Eelco Dolstra et al. in [Imposing a Memory Management Discipline on Software Deployment](https://edolstra.github.io/pubs/immdsd-icse2004-final.pdf) (2004), elaborated in his PhD thesis [The Purely Functional Software
Deployment Model](https://edolstra.github.io/pubs/phd-thesis.pdf) (2006), and further refined by Andrey Mokhov et al. in [Build Systems à la Carte](https://www.microsoft.com/en-us/research/uploads/prod/2018/03/build-systems.pdf) (2018).
