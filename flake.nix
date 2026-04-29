{
  description = "fyers-rs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    claude-code = {
      # SECURITY: Pin to a specific rev for production use
      # url = "github:sadjow/claude-code-nix/<rev>";
      url = "github:sadjow/claude-code-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    dev-template = {
      url = "github:SPRAGE/dev-template";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, claude-code, dev-template, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            claude-code.packages.${system}.default
            pkgs.nodejs
            # TODO: add project dependencies
          ];

          shellHook = ''
            # Auto-sync skills from dev-template
            _src="${dev-template}/template/.claude/skills"
            _dst="$PWD/.claude/skills"
            if [ -d "$_src" ]; then
              mkdir -p "$_dst"
              _n=0
              for _d in "$_src"/*/; do
                [ -d "$_d" ] || continue
                _s=$(basename "$_d")
                if [ ! -d "$_dst/$_s" ] || ! diff -rq "$_src/$_s" "$_dst/$_s" >/dev/null 2>&1; then
                  rm -rf "$_dst/$_s"
                  cp -rL "$_src/$_s" "$_dst/$_s"
                  chmod -R u+w "$_dst/$_s"
                  _n=$((_n + 1))
                fi
              done
              [ "$_n" -gt 0 ] && echo "synced $_n skill(s) from dev-template"
            fi

            # Fix hook permissions (nix flake init strips execute bit)
            if [ -d "$PWD/.claude/hooks" ]; then
              chmod +x "$PWD/.claude/hooks"/*.sh 2>/dev/null || true
            fi

            echo "fyers-rs dev shell ready"
          '';
        };
      }
    );
}
