{
  description = "A flake for Rust development";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";

  outputs = { self, nixpkgs }: {

    devShell.x86_64-linux = with nixpkgs.legacyPackages.x86_64-linux; mkShellNoCC {
      packages = with pkgs; [
        wasm-pack
      
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

    devShell.aarch64-darwin = with nixpkgs.legacyPackages.aarch64-darwin; mkShellNoCC {
      packages = with pkgs; [
        wasm-pack
        yarn
        
        rustc
        rustup 
        cargo
        rustfmt
        rust-analyzer
        clippy
      ];
    };

    shellHook = ''
      export RUSTFLAGS="-C linker=lld"
    '';

  };
}

