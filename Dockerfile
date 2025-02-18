FROM docker.io/itzg/minecraft-server:latest

RUN apt-get update && \
    apt-get install -y build-essential rustup && \
    rustup default stable

COPY . /opt/lazytcp
WORKDIR /opt/lazytcp

RUN cargo install --path .

ENTRYPOINT ["/opt/lazytcp/scripts/lazytcp-startup.sh"]
