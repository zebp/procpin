# procpin

A daemon for pinning processes to specific Zen CCXes.

## Running

### systemd

Running under `systemd` is the same as any other service, simply start and enable the service.

```bash
systemctl enable procpin.service
systemctl start procpin.service
```

## Configuration

To configure the procpin daemon create a `procpin.toml` file in `/etc`.

### Sample configuration

```toml
[programs]
hx = 0 # Pin Helix to CCX 1
ffmpeg = [2, 3] # Pin ffmpeg to either CCX 3 or 4.
```

## Why would I want to pin processes?

AMD's Zen series of architectures group cores into core complexes (commonly referred to as CCXes) with a shared L3 cache. When the scheduler moves processes to a different CCX there can be a performance penalty as theres higher latency to access L3 cache in the other complex, which can be critical for games. On top of that, starting with Zen 2 processors are built using multiple chips per socket called "chiplets", if a process moves between chiplets there is an even larger latency penalty and performance can be drastically impacted.

