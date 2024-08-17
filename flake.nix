{
  description = "A flake for Rust development";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";

  outputs = { self, nixpkgs }: {

    devShell.x86_64-linux = with nixpkgs.legacyPackages.x86_64-linux; mkShellNoCC {
      packages = with pkgs; [
        nodejs_22
        lld_18
        wasm_pack
        rustc
        cargo
        rustfmt
        rust-analyzer
        clippy
        gcc_multi
      ];

      shellHook = ''
        export RUST_BACKTRACE=1
        export RUST_LOG=trace
      '';
    };
  };
}
