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
              printf '\\nTeam development shell\\n'
              printf 'Node: %s\\n' "$(node --version)"
              printf 'npm:  %s\\n' "$(npm --version)"
              printf 'Just: %s\\n\\n' "$(just --version)"
              ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
                export PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH="${pkgs.chromium}/bin/chromium"
              ''}
            '';
          };
        });
    };
}
