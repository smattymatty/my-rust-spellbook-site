---
title: "Micro: The Terminal Editor Between Nano and Vim"
published_at: 2026-02-18
description: "Why I switched from nano and vim to Micro for editing config files on VPSes. A terminal text editor with sane keybindings, syntax highlighting, and zero learning curve."
quote: I respect vim users. I just don't want to be one.
tags:
  - tools
  - terminal
  - editor
---

I spend a lot of time SSHed into VPSes. Editing Docker Compose files, tweaking Caddyfiles, adjusting config files, reading logs. The terminal is where I live when I'm deploying things.

For a while, I thought the only choice was Nano and Vim. Nano is fine - it opens, you type, you save. But it feels like Notepad in 1995. No syntax highlighting worth mentioning, no multi-cursor, no real editing power. You outgrow it fast.

And the keybindings are unhinged. `^` means Ctrl. `M-` means Alt - not "Meta", not some special key, just Alt, but they call it Meta because apparently clarity is optional. Ctrl+X to quit - not Ctrl+Q like every other program on earth. Ctrl+O to save - not Ctrl+S. Ctrl+K to cut a line, Ctrl+U to paste it - because Ctrl+C and Ctrl+V were too intuitive? Ctrl+W to search - which in every browser and most editors closes the tab. The keybindings feel like they were designed by someone who had never used another program before and never planned to.

Vim is the opposite problem. It's incredibly powerful, but every time I open it my brain has to context-switch into "vim mode." Modal editing, memorizing keybindings, accidentally yanking lines when I just wanted to scroll. I know people who love it. I am not those people. When I'm SSHed into a server at midnight fixing a config, I don't want cognitive overhead. I want to open a file, see what I'm doing, make the change, and get out.

Micro is the middle ground.

## What Is Micro?

[Micro](https://micro-editor.github.io/) is a terminal-based text editor that works the way you already expect. Ctrl+S saves. Ctrl+C copies. Ctrl+V pastes. Ctrl+Q quits. Ctrl+Z undoes. If you've ever used VS Code, Sublime, or literally any GUI editor, you already know Micro's keybindings.

It's a single static binary. No dependencies. Install it on any Linux box:

```bash
apt install micro
```

Or grab the latest version directly:

```bash
curl https://getmic.ro | bash
sudo mv micro /usr/local/bin/
```

{~ alert type="info" ~}
Either works. The curl method gets you the newest release, apt gives you whatever your distro packaged. For editing configs on a VPS, it doesn't matter.
{~~}

Once it's installed, make it your default editor so `git commit`, `crontab -e`, and `sudoedit` all use Micro instead of randomly dropping you into nano or vim:

```bash
echo 'export EDITOR=micro' >> ~/.bashrc
echo 'export VISUAL=micro' >> ~/.bashrc
source ~/.bashrc
```

## Why Not Nano?

Nano gets the job done and usually comes pre-installed. But once you've edited a Docker Compose file in Micro with real syntax highlighting, a visible cursor position, and tab completion - going back to nano feels like editing with a rock.

{~ card ~}
Things Micro has that nano doesn't (or barely does):

- Proper syntax highlighting for basically every language and config format
- Multiple cursors (Ctrl+D, same as VS Code)
- Split panes - edit two files side by side in one terminal
- A built-in plugin system
- Mouse support that actually works - click to place cursor, drag to select
- Find and replace that doesn't feel like it was designed in 1988
{~~}

## Why Not Vim?

I respect vim users. I just don't want to be one.

Every time I try vim, I spend more time thinking about the editor than thinking about the file I'm editing. That's the wrong tradeoff for what I do. I'm not writing code on a VPS - I'm editing configs, tweaking settings, making quick changes. I need an editor that disappears and lets me focus on the content.

Micro does that. I open it, I see the file, I make my change, I save, I leave. Zero mental overhead.

If you write code all day in a terminal, vim might be worth the learning curve. If you're a sysadmin or a solo operator managing infrastructure, Micro gives you 90% of the power at 0% of the cognitive cost.

## The Point

If you SSH into servers and edit files, try Micro. It takes 10 seconds to install and zero seconds to learn. You already know the keybindings.

[micro-editor.github.io](https://micro-editor.github.io/)
