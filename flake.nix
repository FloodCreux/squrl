{
  description = "squrl - A terminal HTTP client built with Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

      rustToolchainFor = system: (pkgsFor system).rust-bin.nightly.latest.default;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          rustToolchain = rustToolchainFor system;
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "squrl";
            version = "0.1.0";
            src = ./.;

            nativeBuildInputs = with pkgs; [
              rustToolchain
              pkg-config
              cmake
            ];

            buildInputs =
              with pkgs;
              [
                openssl
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                pkgs.darwin.apple_sdk.frameworks.AppKit
                pkgs.darwin.apple_sdk.frameworks.Security
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              ];

            env.OPENSSL_NO_VENDOR = 1;
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          rustToolchain = (pkgsFor system).rust-bin.nightly.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
              "clippy"
              "llvm-tools-preview"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs =
              with pkgs;
              [
                rustToolchain
                pkg-config
                cmake
                openssl
                just
                pre-commit
                # Security scanning
                cargo-audit
                cargo-deny
                # Code coverage
                cargo-llvm-cov
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                pkgs.darwin.apple_sdk.frameworks.AppKit
                pkgs.darwin.apple_sdk.frameworks.Security
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              ];

            env.OPENSSL_NO_VENDOR = 1;

            shellHook = ''
              echo "Run 'pre-commit install' to set up git hooks"
            '';
          };
        }
      );
    };
}
