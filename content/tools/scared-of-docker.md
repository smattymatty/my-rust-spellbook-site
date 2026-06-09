---
title: "I Used to Be Scared of Docker. Now I'm Scared to Live Without It."
published_at: 2026-04-21
description: "I was scared of Docker for years. Then I learned it properly and it changed everything about how I build and deploy software."
quote: Most of the tools worth learning feel that way at first.
tags:
  - docker
  - devops
  - self-hosting
  - canadian-digital-sovereignty
  - django
  - tools
---

When I first began learning web development - stumbling through Django tutorials, building little toy projects that lived entirely on my computer and nowhere else - I was terrified of Docker. It felt like something that "real" developers used. Senior engineers at proper companies with DevOps teams and budgets. Not me, a self-taught guy trying to get a form to submit correctly and figure out why my migrations kept breaking.

The Docker documentation didn't help. Containers, images, layers, volumes, networks, registries - it was a whole vocabulary I didn't have yet. I had enough new vocabulary to learn just from Django. So I did what most beginners do: I ignored Docker entirely and deployed my first projects the old fashioned way. SFTP. Raw servers. Manually installing Python, manually configuring Nginx, manually crossing my fingers that the production environment vaguely resembled my computer.

---

## The Click

For a long time I leaned heavily on DigitalOcean's App Platform. Push to GitHub, it deploys. No thinking required. It was fine - until the bill started mattering, until I wanted to do something it didn't support, until I realized I had no idea what was actually running or where. I was paying for abstraction I didn't understand, which meant I was also paying for a ceiling I couldn't see.

Docker is what let me get off it entirely. Once I understood containers I realized I didn't need a managed platform to handle deployment complexity for me - I just needed a VPS and a compose file. I moved everything to my own servers. Full control, significantly lower costs, and for the first time I actually understood what was running in production.

The *it works on my machine* problem shrank dramatically too. When you containerize your application, you're shipping the environment along with the code. The container that runs on your laptop is the same container that runs on your server.

Deployment complexity collapsed into a single command. Before Docker, deploying a Django application meant remembering a sequence of steps: pull the code, install dependencies, run migrations, collect static files, restart Gunicorn, hope Nginx didn't need touching. Miss a step or do them out of order and things break in interesting ways. With Docker Compose, that entire sequence becomes `docker compose up -d`. New team member? `docker compose up -d`. New server? `docker compose up -d`.

But the thing that really hooked me - the thing I didn't expect - was what Docker taught me about infrastructure.

---

## An Unexpected Education

When you start containerizing your applications, you are forced to think explicitly about things you used to take for granted. How does your web server talk to your application server? How does your application server talk to your database? Where does your data actually live? What happens when a process crashes?

These questions exist whether you use Docker or not. Docker just makes them impossible to ignore.

I became genuinely obsessed. I spent hours writing YAML, testing different compose structures, tearing things down and spinning them back up. Want to try Nginx instead of Caddy? Swap it out, `docker compose up`, done. Want to add Redis? Three lines. Want to see what happens if the database container crashes? Kill it and watch. The speed at which you can experiment with completely different infrastructure setups is genuinely staggering when you're used to manually configuring servers. What used to take an afternoon of careful shell work became a five minute exercise.

Six years ago I was trying to get a form to submit. Now I run my own infrastructure. I operate federated social media platforms, self-hosted object storage, reverse proxies, and a growing Canadian cloud business. Docker is the connective tissue holding all of it together. I genuinely don't think that trajectory would have happened without it.

## What It's Done for My Work

Everything I build professionally at Storm Developments runs in Docker. Every service, every database, every reverse proxy. When I provision a new server, the first thing I install - after the basics - is Docker. When I design a new product, the first thing I think about is how it will be containerized.

Storm Buckets, my Canadian-hosted S3-compatible object storage service, runs on Garage - a distributed object storage engine - deployed as a Docker container. The Django application serving the customer dashboard runs in Docker. Caddy, which handles TLS and reverse proxying, runs in Docker. When something goes wrong, I know exactly where to look. When I need to move a service to a new server, it's not a migration, it's a copy.

This is what Docker actually gives you at scale: **predictability**. Not magic. Not zero ops work. Predictability. You know what's running, you know how it's configured, you know how to restart it, and you know that the thing running in production is the same thing you tested locally.

## OK, This Is Where It Gets Technical

If you're new to Docker and want a place to start, here's the mental model that unlocked it for me:

{~ card title="The mental model" ~}
**An image is a recipe. A container is the meal.**

You build an image once (or pull one someone else built). From that image you can run as many containers as you want. The image doesn't change when the container runs. The container is the living, running thing.
{~~}

For a Django application, your `Dockerfile` might look like this:

```dockerfile
FROM python:3.12-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

CMD ["gunicorn", "myproject.wsgi:application", "--bind", "0.0.0.0:8000"]
```

And your `docker-compose.yml` might look like this:

```yaml
services:
  web:
    build: .
    env_file: .env
    depends_on:
      - db

  db:
    image: postgres:16
    volumes:
      - postgres_data:/var/lib/postgresql/data
    env_file: .env

  caddy:
    image: caddy:2
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy_data:/data

volumes:
  postgres_data:
  caddy_data:
```

Three services. One command to start them all. Caddy handles HTTPS automatically. Postgres data persists in a named volume. Your Django app talks to the database over Docker's internal network without exposing anything to the outside world.

That's the foundation I use for everything I build.

## The Honest Truth

{~ alert type="warning" ~}
Docker is not perfect. The learning curve is real - I'm not going to pretend otherwise. Build times can be slow if you're not careful about layer caching. Debugging networking issues inside Docker can be frustrating until you understand how Docker networks work. And if you're building something tiny and personal that only ever needs to run on one machine, it might genuinely be more overhead than it's worth.
{~~}

But for anything that needs to run reliably in production, anything that other people depend on, anything you want to be able to move or scale or hand off - Docker is worth every hour of the learning curve.

The version of me that was scared of Docker would not believe what I'm building now. I try to remember that whenever I hit a new tool that feels too complex and too intimidating and too far outside where I currently am.

Most of the tools worth learning feel that way at first.
