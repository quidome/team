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
          # automation (for UI testing via the Playwright MCP server) is
          # Linux-only here; on Darwin, install a browser separately if needed.
          browserPackages = pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.chromium ];
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              nodejs_22
              just
              postgresql
              git
              jq
            ] ++ browserPackages;

            shellHook = ''
              # Printed to stderr, not stdout: `nix develop --command` is also used to
              # run stdio programs (like the Playwright MCP server) whose stdout must
              # stay free of anything but their own protocol output.
              printf '\\nTeam development shell\\n' >&2
              printf 'Node: %s\\n' "$(node --version)" >&2
              printf 'npm:  %s\\n' "$(npm --version)" >&2
              printf 'Just: %s\\n\\n' "$(just --version)" >&2
              ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
                export PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH="${pkgs.chromium}/bin/chromium"
              ''}
            '';
          };
        });
    };
}
