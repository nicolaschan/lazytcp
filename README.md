# lazytcp 💤

> Start TCP server on-demand when connections are established

## Usage

The motivating use-case is starting a Minecraft server on-demand. This way it can consume almost no resources until needed. Since it operates at the TCP level, pretty much any Minecraft server version and type (vanilla, modded, Paper, plugins, etc.) should work. 

Depending on your server, you may need to change the following options:
- `--stdout-ready-pattern` a substring of a stdout log line to detect when the server is ready
- `--shutdown-stdin-command` the command to send to stop the server

The examples assume you have a Docker volume called `mcdata`:
```bash
docker volume create mcdata
```

```bash
nix run github:nicolaschan/lazytcp -- \
  --command 'docker run -it -p 25566:25566 -e VERSION=1.21.4 -e EULA=true -v mcdata:/data itzg/minecraft-server' \
  --downstream-addr localhost:25566 \
  --listen-addr localhost:25565 \
  --stdout-ready-pattern 'RCON running' \
  --shutdown-stdin-command stop \
  --debounce-time-millis 10000
```

There is also a Docker image based off of [itzg/minecraft-server](https://github.com/itzg/docker-minecraft-server) that can be used to run lazytcp in a container.

```bash
docker run -it -p 25565:25565 \
  -e VERSION=1.21.4 \
  -e EULA=true \
  -v mcdata:/data \
  ghcr.io/nicolaschan/lazytcp-minecraft
```

## Roadmap
- [x] Generic child process downstream
- [ ] Minecraft downstream with server list support
- [ ] QEMU downstream with suspend support
