# Lon

Lock & update Nix dependencies.

## Features

- Only uses SRI hashes
- Supports fixed outputs of `builtins.fetchGit` by using an SRI hash and thus
  enables caching for these sources in the Nix Store.
- Allows overriding dependencies via an environment variable for local
  development
- Leverages modern Nix features (concretely this means Nix >= 2.4 is required)

## Installation

The easiest way to use Lon is directly from Nixpkgs. It is currently available
in the `nixos-unstable` branch and will be included in NixOS releases starting
from 25.05.

You can also invoke it via `nix run github:nikstur/lon`.

```console
$ lon
Usage: lon [OPTIONS] <COMMAND>

Commands:
  init      Initialize lon.{nix,lock}
  add       Add a new source
  update    Update an existing source to the newest revision
  modify    Modify an existing source
  remove    Remove an existing source
  freeze    Freeze an existing source
  unfreeze  Unfreeze an existing source
  help      Print this message or the help of the given subcommand(s)

Options:
  -q, --quiet                  Silence all output
  -v, --verbose...             Verbose mode (-v, -vv, etc.)
  -d, --directory <DIRECTORY>  The directory containing lon.{nix,lock}
  -h, --help                   Print help
  -V, --version                Print version
```

## Usage

Initialize Lon:

```console
$ lon init
Writing lon.nix...
Writing empty lon.lock...
```

Add a new GitHub source:

```console
$ lon add github nixos/nixpkgs master
Adding nixpkgs...
Locked revision: 543931cdbf2b2313479c391d956edb5347362744
Locked hash: sha256-8pTC0OIYD47alDVf2mwSytwARCwoH6IqnUfpyshyQX8=
```

Add a new Git source:

```console
$ lon add git lix https://git.lix.systems/lix-project/lix.git main
Adding lix...
Locked revision: a510d1748416ff29b1ed3cab92ac0ad943b6e590
Locked hash: sha256-IjSu5PnS+LFqHfJgueDXrqSBd9/j9GxAbrFK8F1/Z5Y=
Locked lastModified: 1724864109
```

Git sources also support fetching submodules. Enable it by supplying
`--submodules` to Lon.

You can now access these sources via `lon.nix`:

```nix
let
  sources = import ./lon.nix;
  pkgs = import sources.nixpkgs { };
  lix = import sources.lix;
in
  {
    nix = pkgs.nix;
    lix = lix.packages.x86_64-linux.default;
  }
```

You can update individual sources via `lon update nixpkgs` or all sources via
`lon update`. You can even let Lon create a commit for the updates it performs
via `lon update --commit`. The commit message will list all the updates
performed similar to the way `nix flake update --commit-lock-file` does.

### Overriding a Source for Local Development

You can use environment variables that follow the scheme `LON_OVERRIDE_${name}`
to override a source for local development. Lon will use the path this variable
points to instead of the fetching the locked source from `lon.lock`.

Note that no sanitizing of names is performed by Lon. That's why you should
give your sources names that only contain alphanumeric names.

## Invariants

- Support only few repository hosters: Lon does not aim to support all possible
  repository hosters. It will focus on the most important ones and will as much
  as possible rely on generic protocols (e.g. Git) to find and lock updates.
  GitHub is already an exception to this rule, but because of its ubiquity and
  importance, it is unavoidable.
- No tracking besides Git branches. You can still lock e.g. a specific
  revision, but you will have to update it manually.

## On the Shoulders of Giants

Lon is heavily inspired by [niv](https://github.com/nmattia/niv) and
[npins](https://github.com/andir/npins) and builds on their success.
