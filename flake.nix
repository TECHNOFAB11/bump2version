{
  outputs = {
    flake-parts,
    systems,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devenv.flakeModule
        inputs.nix-gitlab-ci.flakeModule
        inputs.nix-devtools.flakeModule
        inputs.treefmt.flakeModule
      ];
      systems = import systems;
      perSystem = {
        lib,
        pkgs,
        config,
        ...
      }: {
        treefmt = {
          projectRootFile = "flake.nix";
          programs = {
            alejandra.enable = true;
            rustfmt.enable = true;
            taplo.enable = true;
            mdformat.enable = true;
          };
        };
        devenv.shells.default = {
          containers = lib.mkForce {};
          packages = with pkgs; [
            openssl
            bacon
            irust
          ];

          languages.rust = {
            enable = true;
            channel = "stable";
          };

          # prevent global libs from being used (try making impure a bit purer)
          enterShell = ''
            export PKG_CONFIG_PATH="$DEVENV_PROFILE/lib/pkgconfig"
          '';

          env.RUST_LOG = "trace";

          pre-commit.hooks = {
            treefmt = {
              enable = true;
              packageOverrides.treefmt = config.treefmt.build.wrapper;
            };
            clippy.enable = true;
            convco.enable = true;
          };
        };
      };
    };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt.url = "github:numtide/treefmt-nix";
    nix-gitlab-ci.url = "gitlab:TECHNOFAB/nix-gitlab-ci?dir=lib";
    nix-devtools.url = "gitlab:TECHNOFAB/nix-devtools?dir=lib";

    fenix.url = "github:nix-community/fenix";
  };

  nixConfig = {
    extra-substituters = [
      "https://nix-community.cachix.org"
      "https://devenv.cachix.org"
    ];

    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
      "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw="
    ];
  };
}
