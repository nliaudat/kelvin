# CI Integration — Kani Formal Verification

This document describes how to integrate the Kelvin Kani Docker image into
GitHub Actions or any Docker-capable CI system.

## GitHub Actions Integration

### Option 1: Direct Docker Compose (Recommended)

Create `.github/workflows/kani-docker.yml`:

```yaml
name: Kani Formal Verification (Docker)

on:
  push:
    branches: [main]
    paths:
      - 'kelvin-core/**'
      - 'proofs/kani/**'
  pull_request:
    branches: [main]
    paths:
      - 'kelvin-core/**'
      - 'proofs/kani/**'

jobs:
  kani:
    runs-on: ubuntu-24.04
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Build Kani Docker image
        run: docker compose -f docker/docker-compose.yml build

      - name: Run all Kani proofs
        run: docker compose -f docker/docker-compose.yml run kani
```

### Option 2: Direct Docker Run (Faster)

```yaml
jobs:
  kani:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - name: Build Kani image
        run: docker build -f docker/Dockerfile.kani -t kelvin-kani .
      - name: Run Kani proofs
        run: docker run --rm -v $(pwd):/kelvin kelvin-kani
```

### Option 3: Parallel Proof Groups

For faster CI, split proofs across parallel jobs:

```yaml
jobs:
  kani-l0-l1:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - name: Build Kani image
        run: docker build -f docker/Dockerfile.kani -t kelvin-kani .
      - name: Run L0 + L1 proofs
        run: docker run --rm -v $(pwd):/kelvin kelvin-kani --fast

  kani-l2-l3:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - name: Build Kani image
        run: docker build -f docker/Dockerfile.kani -t kelvin-kani .
      - name: Run L2 + L3 proofs
        run: |
          docker run --rm -v $(pwd):/kelvin kelvin-kani bash -c "
            cargo kani -p kelvin-core --harness verify_acceleration_action_reaction --enable-unstable --restrict-vtable &&
            cargo kani -p kelvin-core --harness verify_pipeline_invariants --enable-unstable --restrict-vtable
          "
```

## Self-Hosted Runner Configuration

For GitHub self-hosted runners, ensure:

```yaml
jobs:
  kani:
    runs-on: [self-hosted, linux, x64]
    container:
      image: kelvin-kani:latest
      options: --memory=32g --cpus=16
```

## GitLab CI Integration

```yaml
kani-verification:
  stage: test
  image: docker:27
  services:
    - docker:27-dind
  variables:
    DOCKER_HOST: tcp://docker:2375
  script:
    - docker build -f docker/Dockerfile.kani -t kelvin-kani .
    - docker run --rm -v $(pwd):/kelvin kelvin-kani
  rules:
    - changes:
        - kelvin-core/**/*
        - proofs/kani/**/*
```

## CircleCI Integration

```yaml
version: 2.1
jobs:
  kani:
    machine:
      image: ubuntu-2404:2024.01
    steps:
      - checkout
      - run:
          name: Build Kani Docker image
          command: docker build -f docker/Dockerfile.kani -t kelvin-kani .
      - run:
          name: Run Kani proofs
          command: docker run --rm -v $(pwd):/kelvin kelvin-kani
```

## Cache Strategy

The Docker Compose setup uses a named volume for Cargo cache:

```yaml
volumes:
  kani-cargo-cache:

services:
  kani:
    volumes:
      - kani-cargo-cache:/usr/local/cargo
```

This persists the Rust package registry and compiled dependencies between
runs, reducing subsequent build times from ~30 minutes to ~5 minutes.

## Troubleshooting CI Failures

| Symptom | Cause | Fix |
|:---|:---|:---|
| `OOMKilled` | Container ran out of RAM | Increase `--memory` to 32 GB |
| `Unable to find image` | Docker build failed | Check disk space (>15 GB free) |
| `Proof harness not found` | Kani version mismatch | Use `cargo kani --version` and match the CI runner |
| Timeout after 60 min | Proof taking too long | Split into parallel groups (Option 3) |