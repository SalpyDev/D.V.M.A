
# D.V.M.A  — Digital Visual Matrix Algorithm (v1.0)

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**D.V.M.A (Digital Visual Matrix Algorithm)** is a novel visual cryptographic system integrating A–Z micro-notch matrix encoding, authenticated AEAD symmetric encryption (`ChaCha20-Poly1305`), 2-of-2 Shamir secret sharing over $GF(2^8)$, lossless PNG container embedding, and a mandatory key-reconciliation recovery gate.

---

## 🌟 Overview

D.V.M.A transforms any input document (text, binary, Unicode) into an AEAD-encrypted payload, encodes the ciphertext bytes into a Base26 uppercase A–Z string, maps each letter to its canonical radial micro-notch angle around a circular dot, and renders a machine-readable dot matrix grid in a lossless PNG image.

The 256-bit symmetric master key $K$ is split into two mathematical shares using a 2-of-2 Shamir secret sharing scheme:
- **Share A**: Given directly to the user as a passkey (`DVMA-SHARE-A-...`).
- **Share B**: Packaged into a compliant custom ancillary PNG chunk (`dvMa`).

During recovery, the image alone cannot reveal the document. The user must supply Share A, satisfying the mandatory reconciliation gate, before $K$ is reconstructed and AEAD authenticated decryption is attempted.

---

## 🔒 Governing Security Invariants

1. **CSPRNG Key Generation**: A fresh 256-bit master key $K$ is generated for every operation using OS-backed randomness (`rand::rngs::OsRng`).
2. **Authenticated Symmetric Cipher**: ChaCha20-Poly1305 with a random 96-bit nonce and 128-bit authentication tag.
3. **2-of-2 Secret Sharing**: Key split over $GF(2^8)$ using polynomial $x^8 + x^4 + x^3 + x + 1$ (`0x11b`). Neither share alone yields $K$.
4. **Mandatory Reconciliation Gate**: Key reconstruction, payload decoding, and decryption are blocked until Share A is provided.
5. **Fail-Closed Authenticated Decryption**: Unauthenticated or tampered payloads never release plaintext.
6. **Zeroization**: Key material and plaintext buffers are automatically zeroized in memory when dropped (`zeroize`).

---

## 📐 Micro-Notch Radial Geometry

Each letter 'A' through 'Z' corresponds to a radial micro-notch cutout positioned around a circular dot boundary:
$$\theta = N \times \frac{360^\circ}{26} \approx N \times 13.8461538^\circ$$

- **Position `A`** ($N=0$): Notch at **0.0°** (12 o'clock / UP).
- **Position `B`** ($N=1$): Notch at **13.8462°** (Clockwise).
- **Position `N`** ($N=13$): Notch at **180.0°** (6 o'clock / DOWN).
- **Position `Z`** ($N=25$): Notch at **346.1538°**.

  <img width="2374" height="626" alt="alphabet_key_v2" src="https://github.com/user-attachments/assets/c81002ad-440a-4e16-a629-d1e67950498e" />


---

## 🚀 Installation & Build

### Prerequisites
- Rust 1.75+ / Cargo toolchain.

### Building
```bash
cargo build --release
```

The resulting binary will be located at `target/release/dvma`.

---

## 📖 Usage Examples

### 1. Create a DVMA Encrypted PNG Matrix
Encrypt a document and generate the visual matrix PNG artifact along with the user Share A passkey:

```bash
dvma create --input secret.pdf --output matrix.png
```

**Output:**
```text
🔒 Reading input file: "secret.pdf"
⚡ Encrypting document and generating DVMA matrix PNG...

✅ DVMA PNG created successfully at: "matrix.png"
🔑 USER PASSKEY (SHARE A) - KEEP THIS SAFE:
DVMA-SHARE-A-22A5C354A7C02578010020FCA170BEB442A65415A8844AF32F9417C2A0348851F90E75AD382EA645E9C53DBB6E4F33
⚠️ Loss of Share A makes document recovery IMPOSSIBLE from the image alone!
```

---

### 2. Inspect a DVMA PNG Artifact
Inspect a DVMA PNG container, verify its custom `dvMa` chunk, and display Share B metadata:

```bash
dvma inspect --image matrix.png
```

**Output:**
```text
🔍 Inspecting DVMA PNG file: "matrix.png"
Dimensions: 360x360 pixels
Share B Share Set ID: [22, A5, C3, 54, A7, C0, 25, 78]
Share B Coordinate x: 2
Share B Data Length: 32 bytes
✅ Valid DVMA PNG container with embedded Share B.
```

---

### 3. Recover the Original Document
Supply the DVMA PNG image and Share A passkey to reconstruct $K$, decode the matrix, and perform authenticated decryption:

```bash
dvma recover --image matrix.png --share-a "DVMA-SHARE-A-22A5C35..." --output restored.pdf
```

**Output:**
```text
📥 Ingesting DVMA PNG image: "matrix.png"
⚡ Validating shares and reconstructing master key...
🔍 Decoding matrix and performing AEAD authenticated decryption...

🎉 Document recovered and saved successfully to: "restored.pdf"
```

---

## 🧪 Running Tests

Run unit, integration, and end-to-end tests:

```bash
cargo test
```

Run linter checks and formatting checks:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 📁 Repository Structure

```text
.
├── Cargo.toml
├── README.md
├── LICENSE
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Core library interface
│   ├── core/            # Error types and binary framing layer
│   ├── crypto/          # ChaCha20-Poly1305, CSPRNG, GF(2^8) & Shamir secret sharing
│   ├── codec/           # Base26 byte encoding/decoding
│   ├── geometry/        # Micro-notch radial angle mapping, anchors & matrix decoder
│   ├── container/       # PNG reader/writer with custom ancillary 'dvMa' chunk
│   ├── engine/          # Creation & Recovery state machine pipeline
│   └── ui/              # CLI interface using clap
└── tests/               # Comprehensive integration & end-to-end test suite
```

---

## 📜 License
Licensed under the [MIT License](LICENSE).

