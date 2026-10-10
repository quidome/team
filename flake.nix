{
  description = "Team coordinator tool development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

  outputs = { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          # nixpkgs' chromium isn't reliably buildable on Darwin, so browser
          # automation (Playwright tests and the Playwright MCP server) is
          # Linux-only here; on Darwin, install a browser separately if needed.
          browserPackages = pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.chromium ];
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              # Rust backend
              cargo
              rustc
              clippy
              rustfmt
              rust-analyzer
              sqlx-cli
              # Front end (build and tests only; not used at runtime)
              nodejs_22
              # Database client and tools
              postgresql
              just
              git
              jq
            ] ++ browserPackages;

            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

            shellHook = ''
              # Printed to stderr, not stdout: `nix develop --command` is also used to
              # run stdio programs (like the Playwright MCP server) whose stdout must
              # stay free of anything but their own protocol output.
              printf '\nTeam development shell\n' >&2
              printf 'Rust: %s\n' "$(rustc --version)" >&2
              printf 'Node: %s\n\n' "$(node --version)" >&2
              ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
                export PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH="${pkgs.chromium}/bin/chromium"
              ''}
            '';
          };
        });
    };
}
