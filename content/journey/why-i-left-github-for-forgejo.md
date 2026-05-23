---
title: Why I Left GitHub for Forgejo
published_at: 2026-02-17
description: "I'm building a company about Canadian digital sovereignty. The source code can't live on Microsoft servers. Here's how I set up Forgejo on a $7 Canadian VPS in 30 minutes."
quote: "Page load: 4ms. Template render: 1ms. On a $7 VPS."
tags:
  - self-hosting
  - git
  - forgejo
  - canadian-digital-sovereignty
  - docker
---

I am starting a company that positions ourselves as 'champions of Canadian digital sovereignty' - we are building cloud infrastructure products like object storage and a cloud CMS, all hosted in Canada on Canadian-owned servers. But the source code for those products? That was on GitHub.

GitHub is owned by Microsoft. If we're going to tell Canadian organizations to get their data off American servers, our own company's code can't live on one. So I set up Forgejo.

## What is Forgejo?

Forgejo is a self-hosted Git forge. Think GitHub, but you run it yourself on your own server. It's a fork of Gitea, maintained by a community instead of a corporation. It's written in Go, ships as a single binary or Docker image, and runs on basically nothing — under 512MB of RAM.

It does pull requests, issues, CI/CD (Forgejo Actions, compatible with GitHub Actions), package registries, and everything else you'd actually use day-to-day.

[Forgejo project site](https://forgejo.org)

## Why Self-Host Your Own Git?

This isn't a "GitHub bad" post. GitHub is a great product. I still use it for my personal projects and open source packages. That's fine - those are mine, and GitHub is convenient for discoverability.

But company code is different. If I'm telling Canadian organizations that their data should live on Canadian servers, owned by Canadian companies, then my own company's code should too. I can't sell sovereignty and then push code commits to Microsoft.

There's also the practical argument. GitHub has had outages, they've changed their policies, and they have suspended repos/accounts with little warning. When you self-host, none of that is your problem. Your forge is up when you say it's up. Your repos exist because you say they exist.

And honestly? It just feels right. When I push code to git.stormdevelopments.ca, it goes to a server in Canada, hosted by a Canadian company, running on my infrastructure. That's not a service I'm borrowing from a trillion-dollar American corporation. That's mine.

The whole setup took me about 30 minutes with Docker.

## How I Set It Up

I'm running this on a VPS from canadianwebhosting.com - a Canadian-owned hosting provider based in Kelowna, BC. It's their cheapest plan: $6.95 CAD/month, 1 vCore, 1GB RAM, 20GB storage. That's it. That's the whole server.

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
      - "3000:3000"
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

That's the entire Caddyfile. Caddy handles Let's Encrypt certificates automatically - no certbot, cron jobs, or renewal scripts needed.

### DNS

One A record: `git.stormdevelopments.ca → my VPS IP`. Wait for propagation.

### Security

{~ alert type="warning" ~}
I took a few steps to ensure the VPS is a little more secure - fail2ban, SSH hardening, rate limits, etc. This isn't the blog to go in-depth on that, but it's worth mentioning.
{~~}

### Launch

```bash
cd /opt/forgejo
docker compose up -d
```

Hit https://git.stormdevelopments.ca, run through the setup wizard. I picked SQLite for the database - it's just me and maybe a couple contributors, not a company of 500. Disabled self-registration because this isn't a public forge.

The whole thing was up and serving HTTPS in under a minute after the containers pulled. Page load: 4ms. Template render: 1ms. On a $7 VPS.

## Making It Look Like Yours

Out of the box, Forgejo looks like Forgejo. That's fine for most people, but I wanted visitors to git.stormdevelopments.ca to see Storm Developments branding - not a default install with the Forgejo logo.

Forgejo supports custom templates. You create files in specific paths inside the data volume and they get injected into every page. No forking the codebase, no rebuilding from source.

### File Structure

Everything lives under `forgejo-data/gitea/` inside your Docker volume.

```
forgejo-data/gitea/
├── templates/
│   ├── home.tmpl                          # Full homepage replacement
│   └── custom/
│       ├── header.tmpl                    # Injected at top of every page
│       ├── footer.tmpl                    # Injected at bottom of every page
│       └── extra_links.tmpl               # Extra <meta> tags in <head>
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

`header.tmpl` and `footer.tmpl` get injected on every page automatically. I wrote a nav that matches my main site - same glassmorphism dark bar, same ⚡ logo, same links back to stormdevelopments.ca. The footer is the same one from my main site with hardcoded URLs instead of Django template tags.

{~ alert type="info" ~}
The key thing people miss: your custom CSS doesn't load automatically. You need a `<link>` tag in `header.tmpl` to pull it in, otherwise the CSS file just sits there doing nothing.
{~~}

```html
<link rel="stylesheet" href="/assets/css/custom.css">
<link rel="stylesheet" href="/assets/css/storm-forgejo-theme.css">
```

Forgejo serves anything in `public/` at the root path, so `/assets/css/custom.css` maps to `forgejo-data/gitea/public/assets/css/custom.css`.

### Theme Overrides

The theme CSS is just class overrides — I didn't touch any HTML. Forgejo uses Fomantic UI (a Semantic UI fork), so you're overriding classes like `.ui.segment`, `.ui.button`, `.ui.dropdown .menu`, etc. I mapped everything to my brand palette:

```css
:root {
    --storm-electric: #00d4ff;
    --storm-primary: #1e90ff;
    --storm-dark: #050505;
    --storm-surface: #0a0a0a;
    --storm-border: #2d3748;
}
```

Backgrounds, links, buttons, dropdowns, inputs, repo file trees, README rendering, flash messages - all reskinned with `!important` overrides. It's not the most elegant styling solution in the world, but it works without having to modify a single line of Forgejo source.

### Replacing the Logo

Forgejo hardcodes `logo.svg` in its templates. You can't just drop a PNG in there and call it done — it loads `logo.svg` specifically. The workaround is to base64-encode your PNG and embed it inside an SVG:

```bash
B64=$(base64 -w0 logo.png)
cat > logo.svg <<EOF
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="256" height="256">
  <image width="256" height="256" xlink:href="data:image/png;base64,${B64}"/>
</svg>
EOF
```

### Custom Homepage

This is the big one. By default, logged-out visitors see Forgejo's generic landing page — "A painless, self-hosted Git service" with feature cards about cross-platform support. That screams "I installed Forgejo." Not what I want.

Drop a `home.tmpl` in `templates/` (not `templates/custom/` - this one replaces the whole page) and you can build whatever you want. Mine has a hero section, sovereignty messaging, project cards for all Storm Developments products, and the company footer. It links to an external `home.css` for maintainability.

The only Forgejo template syntax you need:

```
{{template "base/head" .}}
<!-- your HTML here -->
{{template "base/footer" .}}
```

That gives you Forgejo's `<head>` tag (with all its JS and base CSS) and its closing scripts. Everything in between is yours.

### OpenGraph / Discord Previews

{~ alert type="info" ~}
If you share your forge link on Discord or Mastodon, it'll show Forgejo's default description unless you override it. `extra_links.tmpl` lets you add `<meta>` tags, but Forgejo's own OG tags load first and win. The fix is in `app.ini`:
{~~}

```ini
[ui.meta]
DESCRIPTION = Git hosting by Storm Developments. Repos, issues, CI/CD - never leaves Canada.
```

After all this, git.stormdevelopments.ca doesn't look like a Forgejo install. It looks like a product.

## Was It Worth It?

Yes. Unambiguously.

The total cost is $6.95 CAD/month. The setup took 30 minutes. The branding took a few hours because I'm picky, but you could skip all of that and have a perfectly functional private Git forge in the time it takes to make coffee.

Right now, Storm Forge is private - just our company's code. But it doesn't have to stay that way.

Canada doesn't have a Codeberg. There's no Canadian-hosted, Canadian-owned Git forge where developers can push code and know it stays here. Every Canadian developer on GitHub, GitLab, or Bitbucket is storing their source code on American servers, subject to American law. That's the same problem Storm Cellar aims to solve for object storage - and Storm Forge could solve it for code.

I'm not announcing anything yet. But the infrastructure is there, the branding is there, and the demand is obvious to anyone paying attention to Canadian tech policy right now.

If you're a Canadian developer who cares about where your code lives, keep an eye on git.stormdevelopments.ca.

The forge is lit. We'll see how big the fire gets.
