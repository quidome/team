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
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              nodejs_22
              just
              git
              jq
            ];

            shellHook = ''
              printf '\\nTeam development shell\\n'
              printf 'Node: %s\\n' "$(node --version)"
              printf 'npm:  %s\\n' "$(npm --version)"
              printf 'Just: %s\\n\\n' "$(just --version)"
            '';
          };
        });
    };
}
