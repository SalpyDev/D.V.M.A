# D.V.M.A  — Digital Visual Matrix Algorithm (v1.0)

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**D.V.M.A (Digital Visual Matrix Algorithm)** is a novel visual cryptographic system integrating A–Z micro-notch matrix encoding, authenticated AEAD symmetric encryption (`ChaCha20-Poly1305`), 2-of-2 Shamir secret sharing over $GF(2^8)$, lossless PNG container embedding, and a mandatory key-reconciliation recovery gate.

> 👀 **Want to see it working first?** Jump to the [Live Example](#-live-example-valt-documentation): a real 2,000-word document encrypted into one PNG, with the passkey and the original text so you can verify it yourself.

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
- A recent stable Rust toolchain with Cargo. Run `rustup update stable` to be sure. (Old versions such as Cargo 1.75 cannot read this project's `Cargo.lock` and fail with `lock file version 4 requires -Znext-lockfile-bump`.)

### Building
```powershell
cargo build --release
```

The resulting binary will be located at `target\release\dvma.exe`.

---

## 📖 Usage Examples

### 📂 Before you start: telling D.V.M.A where your files are

D.V.M.A has no config files and no file picker. You point it at your files by typing their **path** after `--input`, `--output` or `--image`. Your files can be in any folder on your computer.

**Examples of paths**

| Style | Example |
|---|---|
| **Full path** (works from anywhere) | `C:\Users\YourName\Documents\homework.txt` |
| **Relative path** (starts from the folder your terminal is in) | `Documents\homework.txt` |
| **File in the current folder** | `homework.txt` |

**Tips**

- **Put the path in quotes** if it has spaces: `--input "C:\Users\Your Name\My Documents\homework.txt"`.
- **Drag and drop:** in most terminals you can drag a file from your file manager into the terminal window and it pastes the full path for you.
- **See which folder you're in:** run `cd`. Relative paths start from there.
- **Running the program:** the examples below write just `dvma`. Either replace it with the full location, for example `.\target\release\dvma.exe`, or install it once with `cargo install --path .` and then run `dvma` from any folder.

---

### 1. Create a DVMA Encrypted PNG Matrix
Encrypt a document and generate the visual matrix PNG artifact along with the user Share A passkey.

```powershell
dvma create --input "C:\Users\YourName\Documents\secret.pdf" --output "C:\Users\YourName\Desktop\matrix.png"
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

> **Copy the passkey right away** and store it somewhere safe. It is shown once, and it is the only way to unlock the image.

---

### 2. Inspect a DVMA PNG Artifact
Inspect a DVMA PNG container, verify its custom `dvMa` chunk, and display Share B metadata.

```powershell
dvma inspect --image "C:\Users\YourName\Desktop\matrix.png"
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
Supply the DVMA PNG image and Share A passkey to reconstruct $K$, decode the matrix, and perform authenticated decryption.

```powershell
dvma recover --image "C:\Users\YourName\Desktop\matrix.png" --share-a "DVMA-SHARE-A-22A5C35..." --output "C:\Users\YourName\Documents\restored.pdf"
```

> **Tip:** if you leave out `--share-a`, D.V.M.A asks you to type or paste the passkey instead.

**Output:**
```text
📥 Ingesting DVMA PNG image: "matrix.png"
⚡ Validating shares and reconstructing master key...
🔍 Decoding matrix and performing AEAD authenticated decryption...

🎉 Document recovered and saved successfully to: "restored.pdf"
```

The recovered file is an exact, byte-for-byte copy of the original, with the same spaces, line breaks and formatting. It is saved to the `--output` path, not printed on screen.

---

### 4. Share an Encrypted Image
To give an encrypted document to someone else:

1. Send them the **PNG file itself** (as an attachment or file transfer, never a screenshot or a service that converts images to JPEG).
2. Send the **passkey separately**, through a different channel.
3. They run `dvma recover` with the PNG and the passkey. Neither one works alone.

---

## 📏 Capacity & Large Documents

Every byte of ciphertext becomes **two dots** (two Base26 letters), and the matrix is capped at a **128 × 128** grid. That sets a hard ceiling:

| Limit | Value |
|---|---|
| Max grid | 128 × 128 cells |
| Usable data cells | 16,376 (16,384 minus 4 anchors and 4 length-header cells) |
| Framing overhead | 38 bytes (22-byte frame header + 16-byte auth tag) |
| **Max input size** | **8,150 bytes** (8,151 is rejected) |

Anything bigger than about 8 KB is rejected with `Payload too large`. For text, compress it first, encrypt the compressed file, and unzip it after recovery:

```powershell
Compress-Archive -Path "C:\Users\YourName\Documents\report.txt" -DestinationPath "C:\Users\YourName\Documents\report.zip"
dvma create  --input "C:\Users\YourName\Documents\report.zip" --output "C:\Users\YourName\Desktop\report_matrix.png"

# Later, to get it back:
dvma recover --image "C:\Users\YourName\Desktop\report_matrix.png" --share-a "DVMA-SHARE-A-..." --output "C:\Users\YourName\Documents\report_restored.zip"
Expand-Archive "C:\Users\YourName\Documents\report_restored.zip" -DestinationPath "C:\Users\YourName\Documents\report_restored"
```

> ℹ️ A compressed file looks like random symbols if you open it directly. That is normal. Unzip it to read the text. Files that are already compressed (`.docx`, `.pdf`, `.jpg`, `.zip`) will not get much smaller.

---

## ⚠️ Common Mistakes

| Problem | Cause and fix |
|---|---|
| `Error: IO error: No such file or directory` | A path is wrong. Check spelling, put paths with spaces in quotes, and make sure the **output folder already exists** (D.V.M.A creates the file, but not folders). |
| `Share A and Share B belong to different key sets` | The passkey doesn't belong to that image. Every `create` makes a brand-new key, so running `create` again with the same `--output` **overwrites the old image and makes the old passkey useless**. Give each document its own image name and keep each passkey with its image. |
| `Payload too large` | Input is bigger than **8,150 bytes**. Compress it first (see [Capacity & Large Documents](#-capacity--large-documents)). |
| `Share CRC32 checksum mismatch` | The passkey was typed or copied wrong. Copy the whole `DVMA-SHARE-A-...` string again, with no missing characters. |
| Recovery fails and `inspect` says there is no Share B | The PNG was re-encoded (screenshot, JPEG/WebP conversion, "Save image as", or an optimizer). Share B lives in a hidden part of the file and is lost this way. Always use the original PNG file. |
| Existing files are replaced | `--output` silently overwrites a file with the same name, for both `create` and `recover`. |

---

## 🧬 Live Example: Valt Documentation

This is a real working demo of D.V.M.A. We wrote a made-up company profile for a fictional tech company called **Valt** (about 2,000 words), encrypted it, and turned it into a single image made of notched dots. Below you can see the image, the passkey that unlocks it, and the original document that is hidden inside it. You can run the recovery yourself and check that you get the exact same text back.

### Valt Matrix

This image **is** the encrypted document. Every dot is one letter, and the position of the small notch cut into each dot tells the decoder which letter it is (see the alphabet key above). Without the passkey, the image looks like a pattern and reveals nothing.

<img width="2784" height="2784" alt="valt_matrix" src="https://github.com/user-attachments/assets/43c98538-22bb-4669-9298-cd266ce1075a" />


### Passkey (Share A)

This is the user's half of the secret key. D.V.M.A splits the encryption key in two: this passkey is one half, and the other half is hidden inside the PNG file itself. You need **both** to decrypt. This demo passkey is published on purpose because the Valt document is fictional. For real documents, keep your passkey private.

```text
DVMA-SHARE-A-E6A7DE2FF5A38CBD010020C4B4654BE266A0D3EA199D10DCED6C0EDBD6251F50BA5390DF216B2ACD99E501F1C812C8
```

### Valt Documentation

This is the original document before encryption, so you can compare it with what you get back after recovery. It is the text that was compressed with gzip, encrypted, and drawn into the dot matrix above.

<details>
<summary>Click to read the full original document</summary>

```text
VALT TECHNOLOGIES - COMPANY DOCUMENTATION
Internal Reference Edition 4.2 (Fictional Company, For Demonstration Only)

1. EXECUTIVE OVERVIEW

Valt Technologies, commonly called Valt, is a fictional technology company headquartered in the imaginary harbor city of Port Merrow. Founded in 2014 by systems engineer Imani Okafor-Reyes and cryptographer Lars Tennyson-Wu, Valt builds infrastructure software that helps organizations store, move, and prove the integrity of sensitive information. The company employs roughly 1,850 people across eleven offices and reports annual recurring revenue of 412 million credits, the internal unit used in this document to keep all figures clearly fictional.

Valt's guiding idea is simple: trust should be something a system demonstrates, not something a customer is asked to assume. Every product the company ships is designed so that a customer can verify, with mathematics and open tooling, that Valt itself cannot read or silently alter their data. The company calls this principle Proof Before Promise, and it appears on the wall of every Valt office.

2. HISTORY

2014 - The Garage Years. Okafor-Reyes and Tennyson-Wu began working on a small encrypted file locker in a rented boathouse. The first prototype, named Dockside, could split a file into encrypted fragments and scatter them across ordinary cloud buckets. No single bucket held anything useful. The founders sold Dockside to nine small accounting firms and used the revenue to hire their first three engineers.

2016 - Seed Round and the Merrow Protocol. Valt raised its seed funding from a coastal investment collective and published the Merrow Protocol, an open specification for fragment-based storage with verifiable integrity proofs. Independent reviewers found two flaws in the draft; Valt fixed both within a week and credited the reviewers publicly, a habit the company has kept ever since.

2018 - Vaultline. The company launched Vaultline, its first enterprise platform. Vaultline introduced policy-driven encryption, where rules such as "never leave this region" or "require two approvers" are enforced by the storage layer itself rather than by application code. Three regional banks became anchor customers.

2020 - Remote Everything. Valt moved to a distributed work model before it was fashionable, shipping the first version of Lantern, a secure messaging and document review tool for distributed teams. Lantern became the fastest growing product in company history, reaching 600 customers in its first eighteen months.

2022 - The Audit Year. Valt invited three competing security firms to attack its production systems for ninety days and published the full, unredacted results. The exercise cost the company a quarter of its planned feature work but earned a reputation for candor that remains its strongest sales asset.

2024 - Ember and the Public Listing. Valt introduced Ember, an analytics engine that computes over encrypted data. The same year, the company listed on the fictional Harborline Exchange under the ticker VLT.

2026 - Current Day. Valt is pursuing its Horizon plan, which focuses on post-quantum cryptography, regional data sovereignty, and a tenfold reduction in the energy cost of every encrypted operation.

3. MISSION AND VALUES

Mission: To make secure systems the easiest systems to build and the hardest systems to compromise.

Valt's five values are practiced rather than merely posted:

Proof Before Promise. Claims about security must be testable by someone outside the company.

Small Blast Radius. Every system is designed so that a single failure damages as little as possible.

Plain Words. Documentation, contracts, and error messages are written so that a tired person at three in the morning can understand them.

Own the Pager. Engineers who build a service also carry responsibility for its reliability.

Leave It Better. Every code change, document, and meeting should leave the next person with less confusion.

4. PRODUCT PORTFOLIO

4.1 Dockside

Dockside remains Valt's entry product, a lightweight encrypted file locker for small teams. It uses client-side encryption, meaning files are encrypted on the user's device before upload. Dockside's recovery system relies on a two-part key model: one portion is held by the customer and another is stored in the file's metadata. Neither portion alone can open a file. Dockside serves about 70,000 small teams and is deliberately kept simple, with fewer than forty configuration options.

4.2 Vaultline

Vaultline is the flagship enterprise platform. It combines encrypted storage, key management, and policy enforcement into a single control plane. Key capabilities include policy-as-code for data handling, hardware-backed key custody, tamper-evident audit logs written to an append-only ledger, and region pinning that keeps regulated data inside chosen jurisdictions. Vaultline customers include banks, hospitals, shipping consortia, and municipal governments across nineteen countries.

4.3 Lantern

Lantern is a secure collaboration suite offering end-to-end encrypted messaging, document review, and signing workflows. Lantern uses disappearing workspaces: a project room can be configured to destroy all of its content, including keys, on a fixed date. Legal teams use this feature to enforce retention schedules without relying on manual cleanup.

4.4 Ember

Ember is the company's analytics engine for encrypted data. It allows an analyst to run approved queries over records without ever seeing individual rows. Ember uses a combination of secure enclaves and statistical noise to protect individuals while still producing accurate aggregate results. Hospitals use Ember to study treatment outcomes across institutions that cannot legally share patient records.

4.5 Beacon

Beacon is a monitoring service that watches for unusual access patterns inside Valt deployments. It learns what normal behavior looks like for each customer and raises alerts when something deviates, such as a service account suddenly reading ten thousand files at midnight. Beacon explains every alert in plain language and links to the exact log entries that triggered it.

5. TECHNICAL ARCHITECTURE

All Valt products share a common foundation called the Keel. The Keel is a small, heavily reviewed core written in a memory-safe language. It performs three jobs: encrypting and decrypting data, splitting and recombining keys, and signing audit records. Because the Keel is small, around forty thousand lines of code, it can be reviewed exhaustively, and each release is accompanied by a reproducible build so that customers can confirm the binary matches the published source.

Around the Keel sit three layers. The Fabric handles storage, distributing encrypted fragments across multiple providers so that no provider holds a complete object. The Compass handles policy, evaluating rules before any operation is allowed to proceed. The Lookout handles observability, collecting metrics and audit events without ever capturing plaintext customer data.

Valt follows a strict separation of duties. Engineers who write code cannot deploy it to production; deployment requires a second engineer and an automated verification step. Production access is granted through short-lived credentials that expire after fifteen minutes, and every session is recorded and reviewable by the customer on request.

6. SECURITY PRACTICES

Valt treats security as a continuous discipline rather than a certification to be collected. The company maintains a standing bug bounty with payments ranging from five hundred to two hundred fifty thousand credits. It runs quarterly red-team exercises in which a dedicated internal group attempts to breach the company without warning. Findings are tracked publicly in a quarterly transparency report.

Key management follows the principle of least privilege. Master keys are generated inside hardware modules and never exist in plaintext outside them. Customers can choose to hold their own root keys, in which case Valt operationally cannot decrypt their data even under legal compulsion. The company publishes a warrant canary each quarter, stating that it has received no secret orders compelling it to weaken its systems.

Valt's incident response policy commits the company to notify affected customers within twelve hours of confirming any security event, even when the full scope is not yet known. Post-incident reviews are blameless in format and are shared with customers in full.

7. LEADERSHIP

Imani Okafor-Reyes serves as Chief Executive Officer. A former infrastructure engineer for a regional telecom, she is known for insisting that every executive spend one week each year on customer support duty.

Lars Tennyson-Wu serves as Chief Technology Officer and chairs the internal Cryptography Review Board. He holds the company's only veto over changes to the Keel.

Priya Valdecasas leads Product and previously ran the Lantern team. She is credited with the company's discipline of shipping fewer features with higher polish.

Tobias Ngwenya-Harlow is Chief Security Officer and leads the red team, bug bounty, and transparency reporting programs.

Marguerite Aldous-Cho is Chief Financial Officer and joined Valt in 2021 to prepare the company for its public listing.

The board of directors includes five independent members, one of whom must always hold a background in civil liberties or privacy law, a requirement written into the company charter.

8. CULTURE AND PEOPLE

Valt operates a distributed model with offices serving as optional gathering places rather than mandatory workplaces. Core collaboration hours run four hours per day, and the rest of each schedule is left to individual teams. Engineers rotate through support, security review, and documentation duty so that no one drifts away from the real experience of the product.

Compensation bands are published internally, and the company runs an annual pay-equity audit conducted by an outside firm. Every employee receives a learning stipend, and the company sponsors a yearly Open Week in which staff may work on any project of their choosing, provided they share the results. Several Valt products, including Beacon, began as Open Week experiments.

Valt maintains a firm no-surprise policy for layoffs, reorganizations, and strategy changes: significant announcements are made to all employees at the same time, and managers are briefed only an hour earlier.

9. CUSTOMERS AND IMPACT

Valt serves more than 9,400 organizations. Notable fictional customers include the Northern Shipping Consortium, which uses Vaultline to protect cargo manifests across forty ports; the Alderwick Regional Hospital Network, which uses Ember to study rare disease treatments; and the City of Tamsin Falls, which uses Lantern for confidential council deliberations.

Customer satisfaction is measured quarterly, and Valt publishes its retention rate, currently ninety-six percent. The company's customer advisory council meets twice a year and has direct influence on the product roadmap, including the decision to open source the Merrow Protocol reference tools.

10. ENVIRONMENTAL AND SOCIAL COMMITMENTS

Valt has committed to reducing the energy cost of every encrypted operation tenfold by 2030. It buys renewable energy certificates for all of its cloud usage and publishes a yearly carbon report. The company donates two percent of profit to digital rights organizations and offers free Dockside licenses to journalists, human rights groups, and domestic violence shelters.

11. THE HORIZON PLAN

The Horizon plan defines Valt's strategy through 2030 and rests on four pillars.

Post-Quantum Readiness. All Valt products will support hybrid classical and quantum-resistant encryption by the end of 2027, with automatic migration tooling for existing customers.

Data Sovereignty. Valt will expand regional deployments so that customers in any supported country can keep data, keys, and operations entirely within national borders.

Quiet Computing. The company will invest in efficient algorithms and specialized hardware to cut energy use and make strong encryption practical on low-power devices.

Open Verification. Valt will release a customer-facing verification toolkit allowing anyone to independently check storage integrity, policy enforcement, and audit logs without trusting the company.

12. RISKS AND OPEN CHALLENGES

Valt openly acknowledges several risks. Concentration of expertise in a small cryptography group creates a key-person dependency that the company is addressing through apprenticeship programs. Regulatory divergence between jurisdictions increases operational complexity. Competitors with larger sales organizations may outpace Valt in markets where buyers prioritize price over verifiability. Finally, the long-term security of any cryptographic system depends on assumptions that may not hold forever, which is why Valt invests heavily in agility, the ability to replace algorithms quickly without disrupting customers.

13. FREQUENTLY ASKED QUESTIONS

Can Valt read my data? No. With client-side encryption and customer-held root keys, Valt does not possess the information needed to decrypt it.

What happens if I lose my key? If you hold the only copy of your root key, your data cannot be recovered. This is a deliberate trade-off, and Valt strongly recommends the split-key recovery option for important data.

Is Valt open source? The Merrow Protocol, the Keel's cryptographic core, and the verification toolkit are open source. Application layers are proprietary but are audited annually by independent reviewers.

How do I report a vulnerability? Send a report through the bug bounty portal. Valt acknowledges every report within one business day and never pursues legal action against good-faith researchers.

14. CLOSING NOTE

Valt exists because the founders believed that privacy and usability were never truly opposed; they were only badly implemented. Twelve years later, that belief remains the company's compass. This document is a fictional creation intended for demonstration purposes, and every name, product, figure, and place described here is invented.

END OF DOCUMENT
```

</details>

### Good to know

- **Keep the PNG lossless.** Do not convert it to JPEG, resize it, screenshot it, or run it through an image optimizer. Any of these destroys the notches or deletes Share B (the hidden `dvMa` chunk), and the document becomes unrecoverable.
- **Both halves are required.** The image holds the ciphertext and Share B. You hold Share A. Neither alone reveals anything.
- **Tampering is detected.** If any dot is altered, authenticated decryption fails and no plaintext is released.

---

## 🧪 Running Tests

Run unit, integration, and end-to-end tests:

```powershell
cargo test
```

Run linter checks and formatting checks:

```powershell
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

