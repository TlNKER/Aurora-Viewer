# Installing Highland on Linux

Highland is a personal build of [Aurora Viewer](https://github.com/Aurora-Viewer/Aurora-Viewer),
the Second Life viewer. On Linux it installs exactly like Aurora: an archive
to extract and run, with nothing to install as root. It can sit next to an
official Aurora: they keep separate settings, menu entries and icons.

## Requirements

- A 64-bit PC (x86-64) and a graphics card with a **Vulkan** driver.
- A recent distribution: Highland is built on Ubuntu 24.04 and needs glibc
  2.39 or later (Arch, CachyOS, Manjaro, Fedora 40 or later, Ubuntu 24.04 or
  later, Debian 13…).
- X11 or Wayland, and PipeWire, PulseAudio or ALSA for the sound.

## 1. Install the system packages

Once, the same packages as Aurora's. Copy the line for your distribution
into a terminal.

**Arch Linux, CachyOS, Manjaro**

```bash
sudo pacman -S --needed vulkan-icd-loader alsa-lib libxkbcommon wayland libx11 libxcursor libxrandr libxi \
  gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav \
  nss nspr at-spi2-core libcups libxcomposite libxdamage mesa pango cairo
```

**Debian, Ubuntu**

```bash
sudo apt install libvulkan1 libasound2t64 libxkbcommon0 libwayland-client0 libx11-6 libxcursor1 libxrandr2 libxi6 \
  gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav \
  libnss3 libatk-bridge2.0-0t64 libcups2t64 libxcomposite1 libxdamage1 libgbm1 libpango-1.0-0 libcairo2
```

On another distribution, install the same libraries under the names it
gives them.

Plus the Vulkan driver of your graphics card if it is not there yet:
`vulkan-radeon` (AMD), `vulkan-intel` (Intel) or the NVIDIA driver on Arch;
`mesa-vulkan-drivers` or the NVIDIA driver on Debian and Ubuntu.

Saved passwords are kept in your desktop's password store (GNOME Keyring or
KWallet), which most desktops already run.

## 2. Download and extract

Download the newest Highland, and its checksum, into the same folder:

- <https://github.com/TlNKER/Aurora-Viewer/releases/latest/download/Highland-linux-x64.tar.gz>
- <https://github.com/TlNKER/Aurora-Viewer/releases/latest/download/Highland-linux-x64.tar.gz.sha256>

Then, in a terminal in that folder:

```bash
sha256sum -c Highland-linux-x64.tar.gz.sha256
tar -xzf Highland-linux-x64.tar.gz
```

The first command should answer `OK`. The archive holds one folder,
`Highland-linux-x64`, with the viewer (`highland`), its desktop entry and
icon, its licences and this guide. Move it where you like, for example
`~/.local/opt/highland`.

## 3. Start Highland

```bash
cd Highland-linux-x64
./highland
```

To start it from your desktop's menu, with its icon in the panel and the
dock, install the desktop entry and the icon from its folder:

```bash
install -Dm644 highland.png ~/.local/share/icons/hicolor/128x128/apps/highland.png
mkdir -p ~/.local/share/applications
exe=$(printf '%s' "$PWD/highland" | sed -e 's/[\\"`$]/\\&/g' -e 's/\\/\\\\/g' -e 's/%/%%/g')
EXEC="\"$exe\"" awk '/^Exec=/ { print "Exec=" ENVIRON["EXEC"]; next } 1' highland.desktop > ~/.local/share/applications/highland.desktop
```

Run the last two lines again if you move the folder. Avoid a `%` in the
folder's path: GNOME and other GLib desktops then refuse the entry.

## Web pages and video

The first time a web page or a video needs them, Highland asks before it
downloads the media plugins (about 200 MB), once. If video does not play,
check that the GStreamer packages of step 1 are installed (`gst-libav` /
`gstreamer1.0-libav`).

## A new version

Highland does not update itself. When there is a new one, download it again
from the same links (they always give the newest) and extract it in place of
the old folder. Your settings, cache and logs are kept: they are not in the
viewer's folder.

| What | Where |
|---|---|
| Settings | `~/.config/highland` |
| Cache | `~/.cache/highland` |
| Logs, media plugins | `~/.local/share/highland` |

## Second Life links

Preferences › Language and updates › "Open SLURL and Hypergrid links in
Highland" makes Highland open `secondlife://` links for you. Turning it off
gives them back.

## Something wrong?

Tell the person who sent you Highland, with your distribution and graphics
card. It is not the official Aurora: please do not report Highland problems
to the Aurora team.

Highland is free software under the GNU GPL 3 or later, like Aurora; its
source is at <https://github.com/TlNKER/Aurora-Viewer/tree/my-aurora>.
