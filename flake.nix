{
  description = "Rucola - terminal-based markdown note manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Modern Rust build framework with split dependency / workspace caching.
    crane.url = "github:ipetkov/crane";

    # Unified formatter integration (runs on Nix files only here - the Rust
    # toolchain handles Rust formatting through `cargo fmt` in the dev shell).
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Security advisory database consumed by the cargo-audit check.
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = inputs@{ self, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;

      imports = [
        inputs.treefmt-nix.flakeModule
      ];

      # Flake-wide outputs. Exposing an overlay lets downstream flakes pull
      # rucola in via `overlays = [ rucola.overlays.default ]`.
      flake = {
        overlays.default = _final: prev: {
          rucola = self.packages.${prev.stdenv.hostPlatform.system}.rucola;
        };
      };

      perSystem = { config, system, lib, ... }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          # Toolchain exactly pinned. `nix flake update` won't silently bump
          # the Rust compiler - only an explicit edit of this version does.
          rustToolchain = pkgs.rust-bin.stable."1.94.1".default.override {
            extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          };

          # Crane bound to the pinned toolchain. All crane-produced
          # derivations (buildPackage, cargoClippy, cargoNextest, ...) share
          # this single rustc/cargo.
          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

          cargoToml = lib.importTOML ./Cargo.toml;

          # Minimal source: only the inputs cargo actually needs. Editing
          # README.md, CI configs or the flake itself no longer busts
          # rucola's build cache.
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./build.rs
              ./src
              ./tests
              ./default-config
            ];
          };

          # Native tools + C libraries rucola's dependency graph links
          # against. git2 is the driver; the rest of the tree is pure Rust.
          nativeBuildInputs = [ pkgs.pkg-config ];

          buildInputs = with pkgs;
            [ openssl zlib libgit2 ]
            ++ lib.optionals stdenv.hostPlatform.isDarwin (
              [ libiconv ]
              ++ (with pkgs.darwin.apple_sdk.frameworks; [
                CoreFoundation
                Security
                SystemConfiguration
              ])
            );

          # Shared args for every crane invocation in this flake. Using the
          # same set for buildDepsOnly, buildPackage and every check means
          # the fetched-and-built dependency graph is cached exactly once
          # and reused for linting, testing and the final binary.
          commonArgs = {
            inherit src nativeBuildInputs buildInputs;
            pname = cargoToml.package.name;
            version = cargoToml.package.version;

            strictDeps = true;

            # Prefer system libraries over the copies vendored by the -sys
            # crates. Keeps the closure small and inherits upstream patches.
            LIBGIT2_NO_VENDOR = "1";
            OPENSSL_NO_VENDOR = "1";
          };

          # Build only the dependency graph. This is the expensive step
          # (~30s of compile time) and is cached independently from the
          # workspace itself.
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          rucola = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;

            # Tests run in a dedicated nextest check below. Skipping them
            # here keeps `nix build` focused on producing the binary.
            doCheck = false;

            meta = {
              description = lib.strings.removeSuffix "." cargoToml.package.description;
              homepage = cargoToml.package.homepage;
              changelog = "${cargoToml.package.homepage}/blob/main/CHANGELOG.md";
              license = lib.licenses.gpl3Only;
              mainProgram = "rucola";
              platforms = lib.platforms.unix;
            };
          });
        in
        {
          # Share the overlay-augmented pkgs with sibling flake-parts
          # modules (treefmt-nix) so there is exactly one nixpkgs instance.
          _module.args.pkgs = pkgs;

          packages = {
            default = rucola;
            rucola = rucola;
          };

          apps.default = {
            type = "app";
            program = lib.getExe rucola;
          };

          devShells.default = pkgs.mkShell {
            inherit nativeBuildInputs buildInputs;

            packages = with pkgs; [
              rustToolchain
              # Cargo ergonomics
              cargo-edit
              cargo-watch
              cargo-outdated
              cargo-audit
              cargo-nextest
              # Standalone TOML formatter referenced in the comments below
              # (treefmt deliberately doesn't wire it in as a flake check).
              taplo
              # Nix ergonomics
              nil
              config.treefmt.build.wrapper
            ];

            env = {
              LIBGIT2_NO_VENDOR = "1";
              OPENSSL_NO_VENDOR = "1";
              RUST_BACKTRACE = "1";
              RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
            };

            shellHook = ''
              echo "rucola dev shell - $(rustc --version)"
            '';
          };

          # `nix flake check` runs the full matrix: package build, strict
          # clippy, the full test suite via nextest, and a RUSTSEC audit
          # against the pinned advisory-db snapshot.
          checks = {
            inherit rucola;

            clippy = craneLib.cargoClippy (commonArgs // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets --all-features -- -D warnings";
            });

            nextest = craneLib.cargoNextest (commonArgs // {
              inherit cargoArtifacts;
              # test_viewing canonicalises paths that don't exist inside
              # the sandbox; everything else runs in the sandbox cleanly.
              cargoNextestExtraArgs = "--no-fail-fast -E 'not test(=test_viewing)'";
              partitions = 1;
              partitionType = "count";
            });

            audit = craneLib.cargoAudit {
              inherit src;
              inherit (inputs) advisory-db;
            };
          };

          # `nix fmt` formats the Nix files in this repo. Rust and TOML
          # formatting is deliberately not wired in here because it would
          # rewrite upstream-owned source outside this flake's scope -
          # run `cargo fmt` / `taplo fmt` from the dev shell if desired.
          treefmt = {
            projectRootFile = "flake.nix";
            programs.nixpkgs-fmt.enable = true;
          };
        };
    };
}
