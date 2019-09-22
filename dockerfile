FROM rust:1.35

RUN apt-get update && \
    apt-get --no-install-recommends --yes install \
    clang \
    libclang-dev \
    llvm-dev \
    libncurses5 \
    libncursesw5 \
    cmake \
    git

ENV RUST_BACKTRACE=1

# install the node and wallet
RUN git clone --branch v2.0.0 https://github.com/mimblewimble/grin.git
RUN cargo install --path grin
RUN git clone --branch v2.0.0 https://github.com/mimblewimble/grin-wallet.git
RUN cargo install --path grin-wallet

# make bot, wallet, and node directories
RUN mkdir GrinBot
RUN mkdir mywallet
RUN mkdir node

# copy repo into container
COPY . GrinBot

# initialize grin wallet
WORKDIR /mywallet
RUN grin-wallet -p pass init -h

# initialize grin node
WORKDIR /node
RUN grin server config && \
    sed -i -e 's/run_tui = true/run_tui = false/' grin-server.toml

# initialize bot
WORKDIR /GrinBot
RUN chmod u+x scripts/docker_entrypoint.sh
RUN cargo build

# start owner API and bot
CMD scripts/docker_entrypoint.sh
