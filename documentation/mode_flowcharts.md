# Kelvin Cryptosystem — Mode Flowcharts & Sequence Diagrams

## Overview

The Kelvin cryptosystem provides five encryption modes, each with different
security properties and performance characteristics:

| Mode | Cipher | Authentication | Key Derivation |
|------|--------|---------------|----------------|
| **Secure** | ChaCha20 stream cipher | BLAKE3 keyed hash (32-byte tag) | HKDF-SHA512 from orbital seed |
| **Chaos** | Per-step SHAKE256 XOR | None (malleable) | Orbital simulation (1 step per chunk) |
| **Photon** | HKDF→SHAKE256 XOR | None (malleable) | HKDF-SHA512 → SHAKE256 XOF |
| **Quantum** | Hybrid cache+XOR + orbital reseed | None (malleable) | BLAKE3 → SHAKE256 cache + orbital reseed |
| **Prism** | OTP key generator (HE integration) | None (XOR is malleable) | HKDF-SHA512 → SHAKE256 XOF (isolated domain) |

---

## Master Flowchart — Mode Selection

```mermaid
%%{init: {'theme': 'neutral'}}%%
flowchart TD
    A["encrypt/decrypt call"] --> B{mode?}

    B -->|secure| C["process_file_secure / in-memory Secure"]
    C --> D["Read STREAMING_CHUNK_SIZE bytes"]
    D --> E["Zero tag area buf[chunk..chunk+16]"]
    E --> F["k.encrypt/decrypt(&mut buf[..chunk+16])"]
    F --> G["Write buf[..chunk+16] (enc) or buf[..chunk] (dec)"]

    B -->|chaos| H["KelvinStreaming::process_chunk"]
    H --> I{"data.len() > 0?"}
    I -->|Yes| J["Advance simulation by 1 step (Verlet/Euler)"]
    J --> K["Extract keystream via SHAKE256 XOF from orbital state"]
    K --> L["XOR chunk with keystream"]
    L --> M{"More chunks?"}
    M -->|Yes| J
    M -->|No| N["Done"]
    I -->|No| N

    B -->|photon| O["KelvinPhoton::encrypt"]
    O --> P{"reader initialized AND\nbytes_since_reseed < 64 MiB?"}
    P -->|No| Q["ensure_reader(): HKDF + SHAKE256 init + BLAKE3 reseed"]
    P -->|Yes| R["XofReader::read(reader, output) - Persistent SHAKE256"]
    Q --> R
    R --> S["XOR keystream into data"]

    B -->|quantum| T["KelvinQuantum::encrypt"]
    T --> U["keystream_bytes() - bulk cache copy"]
    U --> V{cache exhausted?}
    V -->|Yes| W["refill_keystream_cache(): BLAKE3 → SHAKE256 XOF"]
    V -->|No| X["copy_from cache slice"]
    W --> X
    X --> Y{bytes_since_reseed >= reseed_interval?}
    Y -->|Yes| Z["reseed_from_orbital_chaos(): N Verlet/Euler steps"]
    Y -->|No| AA["XOR into data"]
    Z --> AA
```

---

## Secure Mode — ChaCha20 + BLAKE3 Keyed Authentication

### Architecture

Secure mode uses the ChaCha20 stream cipher for encryption and a BLAKE3 keyed
hash for authentication. The authentication key is derived from the ChaCha20
key+nonce using BLAKE3 key derivation, ensuring the tag is cryptographically
bound to the encryption key.

### Wire Format

```
ciphertext (same length as plaintext) || final_tag (32 bytes)
```

### Flowchart

```mermaid
%%{init: {'theme': 'neutral'}}%%
flowchart TD
    subgraph Initialization
        A1["SecureEncryptor::new(key, nonce)"] --> A2["Create ChaCha20 cipher from key+nonce"]
        A2 --> A3["Derive auth_key = BLAKE3(keyed_hash, key || 'Kelvin Secure Streaming Auth Key' || nonce)"]
        A3 --> A4["Initialize empty plaintext_buf"]
    end

    subgraph Encryption
        B1["update(plaintext, &mut output)"] --> B2{"Already finalized?"}
        B2 -->|Yes| B3["Return error"]
        B2 -->|No| B4["plaintext_buf.extend(plaintext)"]
        B4 --> B5["output.extend(plaintext)"]
        B5 --> B6["ChaCha20.apply_keystream(&mut output[start..])"]
        B6 --> B7["Return Ok"]
    end

    subgraph Finalization
        C1["finalize()"] --> C2{"Already finalized?"}
        C2 -->|Yes| C3["Return error"]
        C2 -->|No| C4["tag = BLAKE3(keyed_hash, auth_key, plaintext_buf)"]
        C4 --> C5["Return 32-byte tag"]
    end

    subgraph Decryption
        D1["SecureDecryptor::new(key, nonce)"] --> D2["Create ChaCha20 cipher (same key)"]
        D2 --> D3["Derive auth_key (same derivation)"]
        D3 --> D4["Initialize empty plaintext_buf"]

        D5["update(ciphertext, &mut output)"] --> D6{"Already finalized?"}
        D6 -->|Yes| D7["Return error"]
        D6 -->|No| D8["output.extend(ciphertext)"]
        D8 --> D9["ChaCha20.apply_keystream(&mut output[start..])"]
        D9 --> D10["plaintext_buf.extend(decrypted bytes)"]
        D10 --> D11["Return Ok (unverified plaintext)"]

        D12["finalize(tag)"] --> D13{"Already finalized?"}
        D13 -->|Yes| D14["Return error"]
        D13 -->|No| D15["expected = BLAKE3(keyed_hash, auth_key, plaintext_buf)"]
        D15 --> D16{"tag == expected?"}
        D16 -->|Yes| D17["Return Ok"]
        D16 -->|No| D18["Zeroize plaintext_buf"]
        D18 --> D19["Return AuthenticationFailed error"]
    end

    A4 --> B1
    B7 --> C1
    D4 --> D5
    D11 --> D12
```

### Sequence Diagram

```mermaid
%%{init: {'theme': 'neutral'}}%%
sequenceDiagram
    participant Caller
    participant SecureEncryptor
    participant SecureDecryptor

    Note over Caller,SecureEncryptor: Encryption
    Caller->>SecureEncryptor: new(key, nonce)
    SecureEncryptor->>SecureEncryptor: Create ChaCha20 cipher
    SecureEncryptor->>SecureEncryptor: Derive auth_key via BLAKE3
    SecureEncryptor-->>Caller: SecureEncryptor instance

    Caller->>SecureEncryptor: update(plaintext, &mut output)
    SecureEncryptor->>SecureEncryptor: plaintext_buf.extend(plaintext)
    SecureEncryptor->>SecureEncryptor: ChaCha20.apply_keystream → ciphertext
    SecureEncryptor-->>Caller: ciphertext appended to output

    Caller->>SecureEncryptor: finalize()
    SecureEncryptor->>SecureEncryptor: BLAKE3(keyed_hash, auth_key, plaintext_buf)
    SecureEncryptor-->>Caller: 32-byte tag

    Note over Caller,SecureDecryptor: Decryption
    Caller->>SecureDecryptor: new(key, nonce)
    SecureDecryptor->>SecureDecryptor: Create ChaCha20 cipher (same key)
    SecureDecryptor->>SecureDecryptor: Derive auth_key (same derivation)
    SecureDecryptor-->>Caller: SecureDecryptor instance

    Caller->>SecureDecryptor: update(ciphertext, &mut output)
    SecureDecryptor->>SecureDecryptor: ChaCha20.apply_keystream → plaintext
    SecureDecryptor->>SecureDecryptor: plaintext_buf.extend(plaintext)
    SecureDecryptor-->>Caller: plaintext appended to output (unverified)

    Caller->>SecureDecryptor: finalize(tag)
    SecureDecryptor->>SecureDecryptor: BLAKE3(keyed_hash, auth_key, plaintext_buf) == tag?
    alt Tag matches
        SecureDecryptor-->>Caller: Ok
    else Tag mismatch
        SecureDecryptor->>SecureDecryptor: Zeroize plaintext_buf
        SecureDecryptor-->>Caller: Err(AuthenticationFailed)
    end
```

### Key Properties

| Property | Value |
|----------|-------|
| **Encryption** | ChaCha20 stream cipher (XOR with keystream) |
| **Authentication** | BLAKE3 keyed hash (32-byte tag) |
| **Key size** | 32 bytes (ChaCha20) |
| **Nonce size** | 12 bytes (IETF ChaCha20 variant) |
| **Tag size** | 32 bytes |
| **Streaming** | Yes — arbitrary chunk sizes via `StreamEncrypt`/`StreamDecrypt` |
| **Forward secrecy** | No — key reuse reveals plaintext |
| **Quantum resistance** | No — ChaCha20 has 128-bit quantum security (Grover's algorithm) |

---

## Chaos Mode — Per-Step SHAKE256 XOR (V2 Streaming)

### Architecture

Chaos mode (V2 Streaming) advances the n-body orbital simulation by one step
for each chunk of data processed. Each step extracts a unique keystream from
the current chaotic orbital state via SHAKE256 XOF, then XORs it with the data.

This is the only mode that uses **true orbital simulation** as the keystream
source, making it the most computationally expensive but also the most
entropy-rich mode.

### Flowchart

```mermaid
%%{init: {'theme': 'neutral'}}%%
flowchart TD
    subgraph Initialization
        A1["KelvinStreaming::new(config, bytes_per_step)"] --> A2["Validate config"]
        A2 --> A3["Clone initial bodies from config"]
        A3 --> A4["Set step=0, allocate keystream_buf[bytes_per_step]"]
    end

    subgraph Process Chunk
        B1["process_chunk(data)"] --> B2{"data empty?"}
        B2 -->|Yes| B3["Return Ok"]
        B2 -->|No| B4["Set offset=0"]

        B5["Take chunk = min(remaining, bytes_per_step)"]
        B5 --> B6["advance_step()"]
        B6 --> B7{"Step multiple of MONITOR_INTERVAL?"}
        B7 -->|Yes| B8["Check for collapse/ejection"]
        B8 --> B9{"Stable?"}
        B9 -->|No| B10["Return StabilityError"]
        B9 -->|Yes| B11
        B7 -->|No| B11["Extract keystream via SHAKE256 XOF from orbital state"]
        B11 --> B12["XOR data[offset..offset+chunk_size] with keystream"]
        B12 --> B13["offset += chunk_size"]
        B13 --> B14{"offset < data.len()?"}
        B14 -->|Yes| B5
        B14 -->|No| B15["bytes_processed += data.len()"]
        B15 --> B16["Return Ok"]
    end

    subgraph Advance Step
        C1["advance_step()"] --> C2{"Integration method?"}
        C2 -->|Verlet| C3["verlet_step(bodies, dt, softening, g)"]
        C2 -->|Euler| C4["euler_step(bodies, dt, softening, g)"]
        C3 --> C5["step += 1"]
        C4 --> C5
    end

    A4 --> B1
    B6 --> C1
```

### Sequence Diagram

```mermaid
%%{init: {'theme': 'neutral'}}%%
sequenceDiagram
    participant Caller
    participant ChaosEncryptor
    participant KelvinStreaming
    participant OrbitalSim

    Note over Caller,OrbitalSim: Encryption / Decryption (XOR is symmetric)

    Caller->>ChaosEncryptor: new(config, bytes_per_step)
    ChaosEncryptor->>KelvinStreaming: new(config, bytes_per_step)
    KelvinStreaming->>KelvinStreaming: Validate config, clone bodies
    KelvinStreaming-->>ChaosEncryptor: KelvinStreaming instance
    ChaosEncryptor-->>Caller: ChaosEncryptor instance

    Caller->>ChaosEncryptor: update(plaintext, &mut output)
    ChaosEncryptor->>ChaosEncryptor: output.extend(plaintext)
    ChaosEncryptor->>KelvinStreaming: encrypt(&mut output[start..])

    loop For each bytes_per_step chunk
        KelvinStreaming->>OrbitalSim: advance_step()
        OrbitalSim->>OrbitalSim: Verlet or Euler integration
        OrbitalSim-->>KelvinStreaming: updated bodies

        KelvinStreaming->>KelvinStreaming: extract_shake256_into(bodies, step, ...)
        KelvinStreaming->>KelvinStreaming: XOR chunk with keystream
    end

    KelvinStreaming-->>ChaosEncryptor: Ok
    ChaosEncryptor-->>Caller: ciphertext appended to output

    Caller->>ChaosEncryptor: finalize()
    ChaosEncryptor-->>Caller: empty Vec (no auth tag)
```

### Key Properties

| Property | Value |
|----------|-------|
| **Encryption** | Per-step SHAKE256 XOR (one simulation step per chunk) |
| **Authentication** | None (malleable) |
| **Keystream source** | Orbital simulation state → SHAKE256 XOF |
| **Chunk size** | Configurable (`bytes_per_step`, default 1 MiB) |
| **Integration** | Verlet (default) or Euler |
| **Stability monitoring** | Every 10,000 steps — checks collapse & ejection |
| **Forward secrecy** | Yes — each step produces unique chaotic state |
| **Quantum resistance** | Yes — SHAKE256 provides 256-bit classical / 128-bit quantum security |
| **Performance** | Slowest mode — each chunk requires an n-body simulation step |

---

## Photon Mode — HKDF→SHAKE256 XOR (V3)

### Architecture

Photon mode (V3) solves the original V1 bottleneck: V1 ran HKDF per 44-byte
key, using only 0.27% of HKDF's capacity. V3 uses HKDF's full capacity to
derive a SHAKE256 XOF seed, then produces arbitrary-length keystream:

```
2048B seed → HKDF-SHA512 → 64B XOF seed → SHAKE256 → unlimited keystream
```

A persistent SHAKE256 XOF reader is kept alive across chunks, only re-created
every 64 MiB via HKDF + BLAKE3 reseed.

### Flowchart

```mermaid
%%{init: {'theme': 'neutral'}}%%
flowchart TD
    subgraph Initialization
        A1["KelvinPhoton::new(seed, max_reseeds)"] --> A2["Store seed, set reseed_count=0"]
        A2 --> A3["reader = None, bytes_since_reseed = 0"]
    end

    subgraph Encrypt
        B1["encrypt(data)"] --> B2{"data empty?"}
        B2 -->|Yes| B3["Return Ok"]
        B2 -->|No| B4["Allocate reusable keystream buffer (up to 1 MiB)"]
        B4 --> B5["Process data in 1 MiB chunks"]
        B5 --> B6["generate_keystream_into(&mut keystream[..chunk_size])"]
        B6 --> B7["XOR chunk with keystream"]
        B7 --> B8{"More chunks?"}
        B8 -->|Yes| B6
        B8 -->|No| B9["bytes_processed += data.len()"]
        B9 --> B10["Return Ok"]
    end

    subgraph Generate Keystream
        C1["generate_keystream_into(output)"] --> C2{"reader is None OR\nbytes_since_reseed >= 64 MiB?"}
        C2 -->|Yes| C3["ensure_reader()"]
        C2 -->|No| C7
        C3 --> C4{"reseed_count >= max_reseeds?"}
        C4 -->|Yes| C5["Return SeedExhausted"]
        C4 -->|No| C6["HKDF-SHA512(seed, reseed_count) → XOF seed"]
        C6 --> C7["SHAKE256 XOF init with xof_seed + domain sep"]
        C7 --> C8["XofReader::read(reader, output)"]
        C8 --> C9["bytes_since_reseed += output.len()"]
        C9 --> C10["Return Ok"]
    end

    subgraph Reseed
        D1["After ensure_reader()"] --> D2["BLAKE3 XOF(domain || seed || reseed_count)"]
        D2 --> D3["Fill new 2048B seed"]
        D3 --> D4["reseed_count += 1"]
        D4 --> D5["bytes_since_reseed = 0"]
        D5 --> D6["Zeroize old XOF seed"]
    end

    A3 --> B1
    B6 --> C1
    C3 --> D1
```

### Sequence Diagram

```mermaid
%%{init: {'theme': 'neutral'}}%%
sequenceDiagram
    participant Caller
    participant PhotonEncryptor
    participant KelvinPhoton
    participant SHAKE256

    Note over Caller,SHAKE256: Encryption / Decryption (XOR is symmetric)

    Caller->>PhotonEncryptor: new(seed, max_reseeds)
    PhotonEncryptor->>KelvinPhoton: new(seed, max_reseeds)
    KelvinPhoton->>KelvinPhoton: Store seed, reader=None
    KelvinPhoton-->>PhotonEncryptor: KelvinPhoton instance
    PhotonEncryptor-->>Caller: PhotonEncryptor instance

    Caller->>PhotonEncryptor: update(plaintext, &mut output)
    PhotonEncryptor->>PhotonEncryptor: Ensure keystream buffer sized
    PhotonEncryptor->>KelvinPhoton: generate_keystream_into(&mut buf)

    alt First call or 64 MiB exhausted
        KelvinPhoton->>KelvinPhoton: ensure_reader()
        KelvinPhoton->>KelvinPhoton: HKDF-SHA512(seed, reseed_count) → XOF seed
        KelvinPhoton->>SHAKE256: Shake256::default() + xof_seed
        SHAKE256-->>KelvinPhoton: Shake256Reader
        KelvinPhoton->>KelvinPhoton: BLAKE3 reseed → new seed
    end

    KelvinPhoton->>SHAKE256: XofReader::read(reader, buf)
    SHAKE256-->>KelvinPhoton: keystream bytes
    KelvinPhoton-->>PhotonEncryptor: Ok

    PhotonEncryptor->>PhotonEncryptor: XOR plaintext with keystream
    PhotonEncryptor-->>Caller: ciphertext appended to output

    Caller->>PhotonEncryptor: finalize()
    PhotonEncryptor-->>Caller: empty Vec (no auth tag)
```

### Key Properties

| Property | Value |
|----------|-------|
| **Encryption** | HKDF→SHAKE256 XOR |
| **Authentication** | None (malleable) |
| **Keystream source** | HKDF-SHA512 → SHAKE256 XOF (persistent reader) |
| **Reseed interval** | 64 MiB |
| **Reseed mechanism** | BLAKE3 XOF (forward secrecy) |
| **Chunk size** | 1 MiB (configurable via `KEYSTREAM_CHUNK_SIZE`) |
| **Forward secrecy** | Yes — BLAKE3 reseed derives fresh seed each interval |
| **Quantum resistance** | Yes — SHAKE256 provides 256-bit classical / 128-bit quantum security |
| **Performance** | Fast — HKDF called only every 64 MiB |

---

## Quantum Mode — Hybrid Cache+XOR + Orbital Reseed (H)

### Architecture

Quantum mode (H) combines the orbital chaos KDF with a quantum-resistant
stream cipher (SHAKE256 XOF). Unlike Photon (which uses deterministic
HKDF→SHAKE256), Quantum periodically refreshes its base seed with **fresh
orbital entropy** via `reseed_from_orbital_chaos`:

```
2048B base seed → BLAKE3 XOF → perturbation → orbital state
  ↓
SHAKE256 XOF → keystream cache (1 MiB)
  ↓
XOR with plaintext/ciphertext
  ↓
Every N bytes: reseed_from_orbital_chaos
  → advance orbital simulation by M steps
  → extract fresh entropy via SHAKE256
  → XOR into base seed
```

### Flowchart

```mermaid
%%{init: {'theme': 'neutral'}}%%
flowchart TD
    subgraph Initialization
        A1["KelvinQuantum::new(seed, max_reseeds)"] --> A2["Perturb orbital state from seed via BLAKE3"]
        A2 --> A3["Allocate keystream_cache[cache_size]"]
        A3 --> A4["Set cache_pos = cache_size (force refill)"]
        A4 --> A5["refill_keystream_cache()"]
    end

    subgraph Refill Cache
        B1["refill_keystream_cache()"] --> B2{"reseed_count >= max_reseeds?"}
        B2 -->|Yes| B3["Return SeedExhausted"]
        B2 -->|No| B4["BLAKE3 XOF(domain || base_seed || reseed_count) → XOF seed"]
        B4 --> B5["SHAKE256 XOF(xof_seed + domain) → fill cache"]
        B5 --> B6["cache_pos = 0, reseed_count += 1"]
        B6 --> B7["Zeroize XOF seed"]
        B7 --> B8["Return Ok"]
    end

    subgraph Encrypt
        C1["encrypt(data)"] --> C2{"data empty?"}
        C2 -->|Yes| C3["Return Ok"]
        C2 -->|No| C4["Allocate reusable keystream buffer (up to 1 MiB)"]
        C4 --> C5["Process data in 1 MiB chunks"]
        C5 --> C6["keystream_bytes(&mut keystream[..chunk_size])"]
        C6 --> C7["XOR chunk with keystream"]
        C7 --> C8{"More chunks?"}
        C8 -->|Yes| C6
        C8 -->|No| C9["Return Ok"]
    end

    subgraph Keystream Bytes
        D1["keystream_bytes(output)"] --> D2["remaining = output.len(), out_pos = 0"]
        D2 --> D3{"remaining > 0?"}
        D3 -->|No| D4["Return Ok"]
        D3 -->|Yes| D5{"cache_pos >= cache.len()?"}
        D5 -->|Yes| D6["refill_keystream_cache()"]
        D5 -->|No| D7["Copy min(remaining, avail) from cache"]
        D7 --> D8["Update cache_pos, total_bytes, bytes_since_reseed"]
        D8 --> D9{"bytes_since_reseed >= reseed_interval?"}
        D9 -->|Yes| D10["reseed_from_orbital_chaos()"]
        D9 -->|No| D11["remaining -= take, out_pos += take"]
        D10 --> D11
        D11 --> D3
    end

    subgraph Orbital Reseed
        E1["reseed_from_orbital_chaos()"] --> E2["For N steps: verlet_step() or euler_step()"]
        E2 --> E3["Extract fresh entropy via SHAKE256 from orbital state"]
        E3 --> E4["XOR fresh entropy into base_seed"]
        E4 --> E5["bytes_since_reseed = 0"]
        E5 --> E6["Zeroize fresh_entropy buffer"]
    end

    A5 --> C1
    C6 --> D1
    D10 --> E1
```

### Sequence Diagram

```mermaid
%%{init: {'theme': 'neutral'}}%%
sequenceDiagram
    participant Caller
    participant QuantumEncryptor
    participant KelvinQuantum
    participant OrbitalState
    participant SHAKE256

    Note over Caller,SHAKE256: Encryption / Decryption (XOR is symmetric)

    Caller->>QuantumEncryptor: new(seed, max_reseeds)
    QuantumEncryptor->>KelvinQuantum: new(seed, max_reseeds)
    KelvinQuantum->>KelvinQuantum: BLAKE3 XOF(seed) → perturb orbital state
    KelvinQuantum->>KelvinQuantum: Allocate keystream cache
    KelvinQuantum->>KelvinQuantum: refill_keystream_cache()
    KelvinQuantum->>SHAKE256: BLAKE3 → SHAKE256 → fill cache
    SHAKE256-->>KelvinQuantum: keystream cache filled
    KelvinQuantum-->>QuantumEncryptor: KelvinQuantum instance
    QuantumEncryptor-->>Caller: QuantumEncryptor instance

    Caller->>QuantumEncryptor: update(plaintext, &mut output)
    QuantumEncryptor->>QuantumEncryptor: Ensure keystream buffer sized
    QuantumEncryptor->>KelvinQuantum: keystream_bytes(&mut buf)

    loop While output not fully filled
        alt Cache exhausted
            KelvinQuantum->>KelvinQuantum: refill_keystream_cache()
            KelvinQuantum->>SHAKE256: BLAKE3 → SHAKE256 → fill cache
            SHAKE256-->>KelvinQuantum: cache refilled
        end

        KelvinQuantum->>KelvinQuantum: Copy from cache to output buffer

        alt Reseed interval reached
            KelvinQuantum->>OrbitalState: reseed_from_orbital_chaos()
            loop N steps
                OrbitalState->>OrbitalState: verlet_step() or euler_step()
            end
            OrbitalState->>OrbitalState: extract_entropy() via SHAKE256
            OrbitalState-->>KelvinQuantum: fresh entropy
            KelvinQuantum->>KelvinQuantum: XOR fresh entropy into base_seed
        end
    end

    KelvinQuantum-->>QuantumEncryptor: keystream buffer filled
    QuantumEncryptor->>QuantumEncryptor: XOR plaintext with keystream
    QuantumEncryptor-->>Caller: ciphertext appended to output

    Caller->>QuantumEncryptor: finalize()
    QuantumEncryptor-->>Caller: empty Vec (no auth tag)
```

### Key Properties

| Property | Value |
|----------|-------|
| **Encryption** | Hybrid cache+XOR + orbital reseed |
| **Authentication** | None (malleable) |
| **Keystream source** | BLAKE3 → SHAKE256 cache (1 MiB default) |
| **Cache refill** | BLAKE3 XOF(domain || base_seed || reseed_count) → SHAKE256 |
| **Orbital reseed** | Every `reseed_interval_bytes` (configurable) |
| **Reseed mechanism** | N Verlet/Euler steps → SHAKE256 extract → XOR into base seed |
| **Orbital steps per reseed** | Configurable (`QUANTUM_DEFAULT_ORBITAL_STEPS`) |
| **Forward secrecy** | Yes — orbital reseed injects fresh chaotic entropy |
| **Quantum resistance** | Yes — SHAKE256 provides 256-bit classical / 128-bit quantum security |
| **Performance** | Moderate — cache refill is fast, orbital reseed is periodic |

---

## Mode Comparison

| Feature | Secure | Chaos | Photon | Quantum | Prism |
|---------|--------|-------|--------|---------|-------|
| **Encryption** | ChaCha20 stream | SHAKE256 XOR | HKDF→SHAKE256 XOR | Cache+XOR + orbital reseed | OTP key generator |
| **Authentication** | BLAKE3 keyed hash (32B) | None | None | None | None |
| **Keystream source** | ChaCha20 | Orbital simulation | HKDF→SHAKE256 | BLAKE3→SHAKE256 + orbital | HKDF→SHAKE256 (isolated) |
| **Forward secrecy** | ❌ | ✅ (per step) | ✅ (per 64 MiB) | ✅ (per reseed interval) | ✅ (per 64 MiB) |
| **Quantum resistant** | ❌ (128-bit) | ✅ | ✅ | ✅ | ✅ |
| **Performance** | Fast | Slowest | Fastest | Moderate | Fastest |
| **Streaming API** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Authenticated wrapper** | Built-in | `KelvinStreamingAuthenticated` | `KelvinPhotonAuthenticated` | `KelvinQuantumAuthenticated` | N/A (OTP only) |

---

## Prism Mode — OTP Key Generator for Homomorphic Encryption

### Architecture

Prism mode (`KelvinPrism`) is a standalone OTP key generator designed for
integration with homomorphic encryption (HE) systems. It wraps `KelvinPhoton`
internally but uses distinct domain separators to ensure Prism-generated OTP
keys are cryptographically isolated from normal V3 Photon keystream.

```
2048B seed → HKDF-SHA512 → 64B XOF seed → SHAKE256 → unlimited OTP keys
```

Each reseed derives a fresh 2048-byte pool via BLAKE3 for forward secrecy.
The domain separators (`DOMSEP_PRISM_KEYSTREAM_V1`, `DOMSEP_PRISM_RESEED_V1`)
ensure cryptographic isolation from V3 Photon.

### Flowchart

```mermaid
%%{init: {'theme': 'neutral'}}%%
flowchart TD
    subgraph Initialization
        A1["KelvinPrism::new(seed, max_reseeds)"] --> A2["Store seed, set reseed_count=0"]
        A2 --> A3["reader = None, bytes_since_reseed = 0"]
    end

    subgraph Generate OTP Key
        B1["generate_otp_key(len)"] --> B2["Allocate output[len]"]
        B2 --> B3["generate_keystream_into(&mut output)"]
        B3 --> B4["Return output"]
    end

    subgraph Split Key
        C1["split_key(len)"] --> C2["Generate K = generate_otp_key(len)"]
        C2 --> C3["Generate random A of length len"]
        C3 --> C4["B = A ⊕ K"]
        C4 --> C5["Return (A, B)"]
        C5 --> C6["Zeroize K"]
    end

    subgraph Encrypt/Decrypt
        D1["encrypt(data) / decrypt(data)"] --> D2{"data empty?"}
        D2 -->|Yes| D3["Return Ok"]
        D2 -->|No| D4["Process data in 1 MiB chunks"]
        D4 --> D5["generate_keystream_into(&mut keystream[..chunk_size])"]
        D5 --> D6["XOR chunk with keystream"]
        D6 --> D7{"More chunks?"}
        D7 -->|Yes| D5
        D7 -->|No| D8["Return Ok"]
    end

    subgraph Generate Keystream
        E1["generate_keystream_into(output)"] --> E2{"reader is None OR\nbytes_since_reseed >= 64 MiB?"}
        E2 -->|Yes| E3["ensure_reader()"]
        E2 -->|No| E7
        E3 --> E4{"reseed_count >= max_reseeds?"}
        E4 -->|Yes| E5["Return SeedExhausted"]
        E4 -->|No| E6["HKDF-SHA512(seed, reseed_count) → XOF seed"]
        E6 --> E7["SHAKE256 XOF init with xof_seed + domain sep (PRISM)"]
        E7 --> E8["XofReader::read(reader, output)"]
        E8 --> E9["bytes_since_reseed += output.len()"]
        E9 --> E10["Return Ok"]
    end

    subgraph Reseed
        F1["After ensure_reader()"] --> F2["BLAKE3 XOF(PRISM domain || seed || reseed_count)"]
        F2 --> F3["Fill new 2048B seed"]
        F3 --> F4["reseed_count += 1"]
        F4 --> F5["bytes_since_reseed = 0"]
        F5 --> F6["Zeroize old XOF seed"]
    end

    A3 --> B1
    A3 --> C1
    A3 --> D1
    B3 --> E1
    D5 --> E1
    E3 --> F1
```

### Sequence Diagram

```mermaid
%%{init: {'theme': 'neutral'}}%%
sequenceDiagram
    participant Caller
    participant KelvinPrism
    participant SHAKE256

    Note over Caller,SHAKE256: OTP Key Generation

    Caller->>KelvinPrism: new(seed, max_reseeds)
    KelvinPrism->>KelvinPrism: Store seed, reader=None
    KelvinPrism-->>Caller: KelvinPrism instance

    Caller->>KelvinPrism: generate_otp_key(256)
    KelvinPrism->>KelvinPrism: generate_keystream_into(&mut buf)

    alt First call or 64 MiB exhausted
        KelvinPrism->>KelvinPrism: ensure_reader()
        KelvinPrism->>KelvinPrism: HKDF-SHA512(seed, reseed_count) → XOF seed
        KelvinPrism->>SHAKE256: Shake256::default() + xof_seed (PRISM domain)
        SHAKE256-->>KelvinPrism: Shake256Reader
        KelvinPrism->>KelvinPrism: BLAKE3 reseed → new seed
    end

    KelvinPrism->>SHAKE256: XofReader::read(reader, buf)
    SHAKE256-->>KelvinPrism: 256 OTP key bytes
    KelvinPrism-->>Caller: OTP key

    Note over Caller,KelvinPrism: Split Key (XOR Homomorphism)

    Caller->>KelvinPrism: split_key(256)
    KelvinPrism->>KelvinPrism: Generate K = generate_otp_key(256)
    KelvinPrism->>KelvinPrism: Generate random A (256 bytes)
    KelvinPrism->>KelvinPrism: B = A ⊕ K
    KelvinPrism->>KelvinPrism: Zeroize K
    KelvinPrism-->>Caller: (A, B) where A ⊕ B = original K

    Note over Caller,KelvinPrism: Static Recryption

    Caller->>KelvinPrism: recrypt(&mut data, &otp_key)
    KelvinPrism->>KelvinPrism: XOR data with otp_key
    KelvinPrism-->>Caller: data XORed in-place
```

### Key Properties

| Property | Value |
|----------|-------|
| **Purpose** | Generate OTP keys for FHE recryption, split-key XOR homomorphism, chaotic FHE keygen |
| **Keystream source** | HKDF-SHA512 → SHAKE256 XOF (persistent reader) |
| **Domain separation** | `DOMSEP_PRISM_KEYSTREAM_V1` / `DOMSEP_PRISM_RESEED_V1` (isolated from V3 Photon) |
| **Reseed interval** | 64 MiB |
| **Reseed mechanism** | BLAKE3 XOF (forward secrecy) |
| **Forward secrecy** | Yes — BLAKE3 reseed derives fresh seed each interval |
| **Quantum resistance** | Yes — SHAKE256 provides 256-bit classical / 128-bit quantum security |
| **Authentication** | None (XOR is malleable) |
| **Key features** | `generate_otp_key()`, `split_key()`, `recrypt()` |

---

## Authenticated Wrappers

All three XOR-based modes (Chaos, Photon, Quantum) can be wrapped with
KMAC128 authentication via the authenticated wrappers:

```mermaid
%%{init: {'theme': 'neutral'}}%%
flowchart LR
    subgraph Authenticated Encryption
        A["plaintext"] --> B["XOR with keystream"]
        B --> C["ciphertext"]
        C --> D["KMAC128(key, ciphertext, custom)"]
        D --> E["32-byte tag"]
        C --> F["wire: ciphertext || version(1B) || tag(32B)"]
        E --> F
    end

    subgraph Authenticated Decryption
        G["wire: ciphertext || version || tag"] --> H["Verify version byte"]
        H --> I["KMAC128(key, ciphertext, custom) == tag?"]
        I -->|Yes| J["XOR with keystream → plaintext"]
        I -->|No| K["Return AuthenticationFailed"]
    end
```

The MAC key is derived from the same 2048-byte seed using HKDF-SHA512 with a
distinct domain separator (`DOMSEP_MAC_KEY_V1`), preventing related-key attacks
between keystream generation and authentication.
