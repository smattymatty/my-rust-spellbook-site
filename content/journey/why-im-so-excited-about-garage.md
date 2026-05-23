---
title: "Why I'm So Excited About Garage: Self-Hosted S3 Storage for My Obsidian Notes"
published_at: 2026-04-07
description: "I needed S3 storage I actually owned, on Canadian hardware. Garage is the answer. Here's how I set up my first node and synced my Obsidian notes to it."
quote: I went looking for something I could actually run myself, on Canadian hardware. I found Garage.
tags:
  - garage
  - self-hosting
  - canadian-digital-sovereignty
  - object-storage
  - obsidian
  - docker
---

I'm building a company around Canadian digital sovereignty. But for a while, I was storing my own S3 data on Backblaze. Free tier, US servers, zero control. Hard to preach sovereignty when you're not living it.

So I went looking for something I could actually run myself, on Canadian hardware. I found Garage.

Garage is a self-hosted, S3-compatible object storage server. Written in Rust, open source, designed to run on commodity hardware - one node to start, many nodes when you're ready. It's the S3-compatible storage you actually own.

I'm going to be writing about it a lot. This is part one - and we're starting small. I set up a single Garage node on a Canadian VPS and connected it to my Obsidian notes. Here's how it went.

## What is Garage, exactly?

Most applications that store files need somewhere to put them. Traditionally that meant a filesystem - folders, paths, the stuff you're used to. But filesystems are complicated to distribute across multiple servers. Object storage is a simpler model. Instead of folders and files, you have buckets and objects. You put something in at a key, you get it back by that key. That's basically it. No folder hierarchies, no POSIX complexity - just put and get.

Amazon introduced S3 (Simple Storage Service) in 2006 and it became the de-facto standard. Thousands of applications now speak "S3" - they know how to talk to an S3-compatible endpoint to store and retrieve files. The protocol is everywhere.

Garage is a self-hosted S3-compatible server. Point any S3-aware application at your Garage instance instead of Amazon, and it just works. Your data stays on your hardware, in your country, under your control.

What makes Garage different from other self-hosted S3 alternatives is how it handles distribution. There is no master node, no leader, no single point of failure. Every node is equivalent. It uses a coordination-free approach that makes it unusually tolerant of high latency between nodes - which means you can run it across geographically distributed servers on commodity hardware without it falling apart.

For my purposes right now, I'm running a single node. No geo-distribution yet - just one VPS in Canada, one Garage instance, one bucket for my Obsidian notes. But the path to a proper multi-node Canadian storage cluster is already built into the design. That's what excites me.

## Getting Started with Garage

I spun up a fresh VPS from Canadian Web Hosting. Once I was in, the setup was surprisingly straightforward.

Garage distributes an official Docker image, which is how I went. If you have Docker and Docker Compose on your server, you are most of the way there.

First, I created a directory for the Garage config and compose file:

```bash
mkdir -p /opt/garage
cd /opt/garage
```

Then I generated a config file. The key things to set are where Garage stores its metadata and data, and a random secret for internal RPC communication between nodes.

From the official documentation:

```bash
cat > garage.toml <<EOF
metadata_dir = "/var/lib/garage/meta"
data_dir = "/var/lib/garage/data"
db_engine = "sqlite"

replication_factor = 1

rpc_bind_addr = "[::]:3901"
rpc_public_addr = "127.0.0.1:3901"
rpc_secret = "$(openssl rand -hex 32)"

[s3_api]
s3_region = "garage"
api_bind_addr = "[::]:3900"
root_domain = ".s3.garage.localhost"

[s3_web]
bind_addr = "[::]:3902"
root_domain = ".web.garage.localhost"
index = "index.html"

[k2v_api]
api_bind_addr = "[::]:3904"

[admin]
api_bind_addr = "[::]:3903"
admin_token = "$(openssl rand -base64 32)"
metrics_token = "$(openssl rand -base64 32)"
EOF
```

A few things worth noting here. The `db_engine` is set to `sqlite` - recommended for single-node deployments. The `s3_region` is `garage`, not `us-east-1` - this trips people up when configuring S3 clients later. And the secrets are generated fresh with `openssl rand`, so every install is unique.

Then a minimal `docker-compose.yml`:

```yaml
services:
  garage:
    image: dxflrs/garage:v2.2.0
    container_name: garaged
    network_mode: host
    restart: always
    volumes:
      - ./garage.toml:/etc/garage.toml
      - /var/lib/garage/meta:/var/lib/garage/meta
      - /var/lib/garage/data:/var/lib/garage/data
```

Garage publishes an official Docker image on Docker Hub under `dxflrs/garage`. You can browse available versions at hub.docker.com/r/dxflrs/garage - at the time of writing, v2.2.0 is the latest stable release. The `image:` line in your compose file tells Docker exactly which version to pull.

Then:

```bash
docker compose up -d
```

That's it. Garage is running.

I added a shell alias so I could use the Garage CLI without typing `docker exec` every time:

```bash
echo "alias garage='docker exec -ti garaged /garage'" >> ~/.bashrc
source ~/.bashrc
```

Then assigned my node a role in the cluster layout - giving it a zone name and declaring its capacity:

```bash
garage layout assign -z canada-1 -c 10G <node_id>
garage layout apply --version 1
```

{~ alert type="info" ~}
The node ID comes from `garage status`. You only need the first 8 characters.
{~~}

One command to check everything is healthy:

```bash
garage status
```

If you see your node listed as healthy with a zone and capacity assigned, you are ready to start creating buckets.

## The Bucket and the Keys

Garage is running, but it's empty. Before any application can store files, you need two things: a bucket to store them in, and an API key with permission to access it.

Create the bucket:

```bash
garage bucket create obsidian-vault
```

Then create an API key:

```bash
garage key create obsidian-key
```

This will output a Key ID and a Secret key.

{~ alert type="warning" ~}
Save both somewhere safe - you will need them in a moment and the secret key is only shown once.
{~~}

Now, for the glue: give the key permission to read and write the bucket:

```bash
garage bucket allow \
  --read \
  --write \
  --owner \
  obsidian-vault \
  --key obsidian-key
```

That's it. You now have a bucket named `obsidian-vault` with a dedicated key that has full access to it. You can verify everything looks right with:

```bash
garage bucket info obsidian-vault
```

You should see your key listed under "Keys for this bucket" with RWO permissions - Read, Write, Owner.

## Exposing Garage to the World

Garage is running on your server, but right now it's only accessible from within the server itself. To connect Obsidian - or any other application - from your local machine, you need to expose it over HTTPS.

Garage has no built-in TLS support. That's by design - it delegates that responsibility to a reverse proxy. I used Caddy, which handles Let's Encrypt certificates automatically with zero configuration.

Add Caddy to your `docker-compose.yml`:

```yaml
services:
  garage:
    image: dxflrs/garage:v2.2.0
    container_name: garaged
    network_mode: host
    restart: always
    volumes:
      - ./garage.toml:/etc/garage.toml
      - /var/lib/garage/meta:/var/lib/garage/meta
      - /var/lib/garage/data:/var/lib/garage/data

  caddy:
    image: caddy:2
    container_name: caddy
    restart: always
    network_mode: host
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - ./caddy-data:/data
      - ./caddy-config:/config
```

Create a `Caddyfile` in the same directory:

```
s3.garage-one.stormdevelopments.ca {
    @anonymous {
        not header Authorization *
        not header X-Amz-Security-Token *
    }
    respond @anonymous 403

    reverse_proxy localhost:3900
}
```

The anonymous block silently drops unauthenticated requests before they reach Garage - no fingerprinting, no information leakage to bots and scanners.

Then add a DNS A record pointing your chosen subdomain at your VPS IP. I used my existing domain:

```
s3.anything.mydomain.ca → vps.ip.add.res
```

Once DNS propagates, bring everything up:

```bash
docker compose up -d
```

Caddy will automatically obtain a Let's Encrypt certificate for your subdomain. You can verify it's working with:

```bash
curl https://s3.garage-one.stormdevelopments.ca
```

You should get a plain 403 - no body, no information. Authenticated S3 requests will pass through normally. Garage is now reachable from anywhere.

## Connecting Obsidian

I used the Remotely Save plugin for Obsidian. It supports S3-compatible storage out of the box, which means it speaks Garage natively - no hacks required.

Install it from the Obsidian community plugins list, then open its settings and choose "S3 or compatible" as the remote service.

Fill in the following:

- **Endpoint:** `https://s3.garage-one.stormdevelopments.ca` (your Caddy-proxied Garage endpoint)
- **Region:** `garage` - not `us-east-1`, not your zone, this is the classic gotcha
- **Access Key ID:** the Key ID from `garage key create`
- **Secret Access Key:** the secret key from `garage key create`
- **Bucket Name:** `obsidian-vault`
- **S3 URL Style:** Path-Style - Garage does not support virtual hosted-style URLs

Hit the check button. If everything is configured correctly you should see "great! this bucket can be accessed!"

Then hit sync. Your notes are now leaving your machine and landing on your own Canadian server.

To verify, check the logs on your Garage node:

```bash
docker compose logs garage --tail 20
```

This is what I saw when I synced my notes for the first time:

```
(key GKxxx) GET /obsidian-vault/?list-type=2
(key GKxxx) PUT /obsidian-vault/Notes%20from%20Garage%20FOSDEM%20Talks.md
(key GKxxx) HEAD /obsidian-vault/Notes%20from%20Garage%20FOSDEM%20Talks.md
```

My notes from studying Garage - from FOSDEM talks *Introducing Garage* (2022), *Advances in Garage* (2024), and *Garage Object Storage: 2.0* (2026) I watched months ago - are now the first files stored on my own Canadian Garage node.

## Want to Set Up Your Own?

{~ card ~}
If you're going to try this and you need a Canadian VPS to run Garage on, ServaRica is the most cost-effective option I've found for storage-heavy hosting. Montreal-based, Hydro-Québec powered, running for self-hosters since 2010, ticket support that actually answers. Here's my referral link: **[clients.servarica.com/aff.php?aff=1318](https://clients.servarica.com/aff.php?aff=1318)**. Small commission to me if you use it.
{~~}

More coming soon.
