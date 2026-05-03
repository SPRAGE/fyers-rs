{
  description = "fyers-rs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
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

  outputs = { self, nixpkgs, fenix, flake-utils, claude-code, dev-template, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfreePredicate = pkg:
            builtins.elem (nixpkgs.lib.getName pkg) [
              "github-copilot-cli"
            ];
        };
        rustToolchain = with fenix.packages.${system}; combine [
          stable.rustc
          stable.cargo
          stable.clippy
          stable.rustfmt
          stable.rust-src
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            claude-code.packages.${system}.default
            pkgs.github-copilot-cli
            pkgs.nodejs
            rustToolchain
            pkgs.rust-analyzer
            pkgs.cargo-audit
            pkgs.cargo-deny
            pkgs.cargo-edit
            pkgs.cargo-nextest
            pkgs.cargo-watch
            pkgs.just
            pkgs.git
            pkgs.pkg-config
            pkgs.openssl
            pkgs.cacert
            pkgs.curl
            pkgs.jq
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

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

            echo "🦀 fyers-rs Rust dev shell ready"
            echo "Rust: $(rustc --version)"
            echo "Cargo: $(cargo --version)"
            echo ""
            echo "Useful commands:"
            echo "  cargo check       # type-check the library"
            echo "  cargo test        # run tests"
            echo "  cargo clippy      # lint"
            echo "  cargo fmt         # format"
            echo "  cargo nextest run # run tests with nextest"
            echo "  copilot --help    # GitHub Copilot CLI"
          '';
        };
      }
    );
}
