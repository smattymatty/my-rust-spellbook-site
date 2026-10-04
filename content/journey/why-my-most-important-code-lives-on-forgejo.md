---
title: Why My Most Important Code Lives on Forgejo
published_at: 2026-02-17
carved: true
description: "My company's code lives on a Forgejo forge I run on Canadian hardware, not on GitHub. Here's the setup and the branding behind Storm Forge."
quote: "That's not a service I'm borrowing from a trillion-dollar American corporation. That's mine."
tags:
  - self-hosting
  - git
  - forgejo
  - canadian-digital-sovereignty
  - docker
---

GitHub is owned by Microsoft, and I wanted the code my company depends on to live on a forge I own. So I set up Forgejo.

## What is Forgejo?

Forgejo is a self-hosted Git forge. Think GitHub, but you run it yourself on your own server. It's a fork of Gitea, maintained by a community instead of a corporation. It's written in Go, ships as a single binary or Docker image, and runs on under 512MB of RAM.

It does pull requests, issues, CI/CD (Forgejo Actions, compatible with GitHub Actions), package registries, and everything else you'd actually use day-to-day.

[Forgejo project site](https://forgejo.org)

## Why Self-Host Your Own Git?

This isn't a "GitHub bad" post. GitHub is a great product. I still use it for my personal projects and open source packages. GitHub is convenient for discoverability.

But company code is different. It's what the business runs on, so it lives on infrastructure I control. It would also be strange to sell sovereignty and push my commits to Microsoft.

There's also the practical argument. GitHub has had outages, they've changed their policies, and they have suspended repos/accounts with little warning. When you self-host, those decisions are yours, and so are the outages. Your repos exist because you say they exist.

When I push code to git.stormdevelopments.ca, it goes to a server in Canada, hosted by a Canadian company, running on my infrastructure. That's not a service I'm borrowing from a trillion-dollar American corporation. That's mine.

## How I Set It Up

I'm running this on a VPS from canadianwebhosting.com, a Canadian-owned hosting provider based in Kelowna, BC. It's their cheapest plan: $6.95 CAD/month, 1 vCore, 1GB RAM, 20GB storage.

### Docker Compose

Forgejo runs as a Docker container. I put Caddy in front of it for automatic HTTPS. The entire deployment is two files in `/opt/forgejo/`:

`docker-compose.yml`:

```yaml
networks:
  forgejo:
    external: false

services:
  forgejo:
    image: codeberg.org/forgejo/forgejo:13
    container_name: forgejo
    environment:
      - USER_UID=1000
      - USER_GID=1000
      - FORGEJO__server__DOMAIN=git.stormdevelopments.ca
      - FORGEJO__server__ROOT_URL=https://git.stormdevelopments.ca/
      - FORGEJO__server__HTTP_PORT=3000
      - "FORGEJO__DEFAULT__APP_NAME=Storm Forge: Canadian Code. Canadian Servers."
    restart: always
    networks:
      - forgejo
    volumes:
      - ./forgejo-data:/data
      - /etc/localtime:/etc/localtime:ro
    ports:
      - "2222:22"

  caddy:
    image: caddy:2
    container_name: caddy
    restart: always
    networks:
      - forgejo
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - ./caddy-data:/data
      - ./caddy-config:/config
```

`Caddyfile`:

```
git.stormdevelopments.ca {
    reverse_proxy forgejo:3000
}
```

That's the entire Caddyfile. Caddy handles Let's Encrypt certificates automatically, with no certbot or renewal cron.

### DNS

One A record: `git.stormdevelopments.ca → my VPS IP`. Wait for propagation.

### Security

{~ alert type="warning" ~}
I took a few steps to ensure the VPS is a little more secure, including fail2ban, SSH hardening and rate limits. This isn't the blog to go in-depth on that, but it's worth mentioning.
{~~}

### Launch

```bash
cd /opt/forgejo
docker compose up -d
```

Hit https://git.stormdevelopments.ca, run through the setup wizard. I picked SQLite for the database, since it's just me and maybe a couple contributors, not a company of 500. Disabled self-registration because this isn't a public forge.

The whole thing was up and serving HTTPS in under a minute after the containers pulled.

## Making It Look Like Yours

Out of the box, Forgejo looks like Forgejo. That's fine for most people, but I wanted visitors to git.stormdevelopments.ca to see Storm Developments branding instead of a default install with the Forgejo logo.

Forgejo supports custom templates. You create files in specific paths inside the data volume and they get injected into every page.

### File Structure

Everything lives under `forgejo-data/gitea/` inside your Docker volume.

```
forgejo-data/gitea/
├── templates/
│   ├── home.tmpl                          # Full homepage replacement
│   └── custom/
│       ├── header.tmpl                    # Injected at the end of <head>
│       ├── footer.tmpl                    # Injected at bottom of every page
│       └── extra_links.tmpl               # Extra links in the top nav
└── public/assets/
    ├── css/
    │   ├── custom.css                     # Nav + footer styles
    │   ├── storm-forgejo-theme.css        # Full theme overrides
    │   └── home.css                       # Homepage-specific styles
    └── img/
        ├── logo.svg                       # Replaces Forgejo logo sitewide
        └── favicon.svg                    # Browser tab icon
```

### The Nav and Footer

`header.tmpl` and `footer.tmpl` get injected on every page automatically. I wrote a nav that matches my main site, with the same glassmorphism dark bar, ⚡ logo and links back to stormdevelopments.ca. The footer is the same one from my main site with hardcoded URLs instead of Django template tags.

{~ alert type="info" ~}
Your custom CSS doesn't load automatically. You need a `<link>` tag in `header.tmpl` to pull it in, otherwise the CSS file just sits there doing nothing.
{~~}

```html
<link rel="stylesheet" href="/assets/css/custom.css">
<link rel="stylesheet" href="/assets/css/storm-forgejo-theme.css">
```

Forgejo serves anything in `public/` at the root path, so `/assets/css/custom.css` maps to `forgejo-data/gitea/public/assets/css/custom.css`.

### Theme Overrides

The theme CSS is just class overrides, and I didn't touch any HTML. Forgejo uses Fomantic UI (a Semantic UI fork), so you're overriding classes like `.ui.segment`, `.ui.button`, `.ui.dropdown .menu`, etc. I mapped everything to my brand palette:

```css
:root {
    --storm-electric: #00d4ff;
    --storm-primary: #1e90ff;
    --storm-dark: #050505;
    --storm-surface: #0a0a0a;
    --storm-border: #2d3748;
}
```

Backgrounds, links, buttons, dropdowns, inputs, repo file trees, README rendering, flash messages are all reskinned with `!important` overrides. It's not the most elegant styling solution in the world, but it works without having to modify a single line of Forgejo source.

### Replacing the Logo

Forgejo hardcodes `logo.svg` in its templates. You can't just drop a PNG in there and call it done, because it loads `logo.svg` specifically. The workaround is to base64-encode your PNG and embed it inside an SVG:

```bash
B64=$(base64 -w0 logo.png)
cat > logo.svg <<EOF
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="256" height="256">
  <image width="256" height="256" xlink:href="data:image/png;base64,${B64}"/>
</svg>
EOF
```

### Custom Homepage

By default, logged-out visitors see Forgejo's generic landing page, "A painless, self-hosted Git service" with feature cards about cross-platform support.

Drop a `home.tmpl` in `templates/` (not `templates/custom/`, since this one replaces the whole page) and you can build whatever you want. Mine has a hero section, sovereignty messaging, project cards for all Storm Developments products, and the company footer. It links to an external `home.css` for maintainability.

The only Forgejo template syntax you need:

```
{{template "base/head" .}}
<!-- your HTML here -->
{{template "base/footer" .}}
```

That gives you Forgejo's `<head>` tag (with all its JS and base CSS) and its closing scripts. Everything in between is yours.

### OpenGraph / Discord Previews

{~ alert type="info" ~}
If you share your forge link on Discord or Mastodon, it'll show Forgejo's default description unless you override it. `header.tmpl` lets you add `<meta>` tags, but Forgejo's own OG tags load first and win. The fix is in `app.ini`:
{~~}

```ini
[ui.meta]
DESCRIPTION = Git hosting by Storm Developments. Repos, issues, CI/CD - never leaves Canada.
```

After all this, git.stormdevelopments.ca looks like a product.

## What It Costs

The total cost is $6.95 CAD/month. The setup took 30 minutes. The branding took a few hours because I'm picky, but you could skip all of that and have a working private Git forge in under an hour.

Storm Forge is private, just our company's code on servers in Canada.
