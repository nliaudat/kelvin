# SMT solvers used as backends by the other tools.
FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        z3 \
        cvc5 \
    && rm -rf /var/lib/apt/lists/*

RUN z3 --version && cvc5 --version

WORKDIR /work
ENTRYPOINT ["bash"]
