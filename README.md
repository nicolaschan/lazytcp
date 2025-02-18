# lazytcp 💤

> Start TCP server on-demand when connections are established

## Usage

The motivating use-case is starting a Minecraft server on-demand. This way it can consume no resources until needed.

```bash
nix run github:nicolaschan/lazytcp -- \
  --command 'docker run -it -p 25566:25566 -e VERSION=1.21.4 -e EULA=true -v mcdata:/data itzg/minecraft-server' \
  --downstream-addr localhost:25566 \
  --listen-addr 127.0.0.1:25565 \
  --stdout-ready-pattern 'RCON running' \
  --shutdown-stdin-command stop \
  --debounce-time-millis 10000
```

