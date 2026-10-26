# Arch source install (AUR-style, no AUR)

Builds Serein on your machine and installs it as a normal pacman package, so
`pacman -R serein` removes it.

```sh
git clone https://github.com/HaraldErik/Serein-Fork
cd Serein-Fork
bash packaging/arch/install.sh
```

Or without cloning first:

```sh
curl -fsSL https://raw.githubusercontent.com/HaraldErik/Serein-Fork/main/packaging/arch/install.sh | bash
```

Run as your normal user; `sudo` is used only for `pacman`. Set `SEREIN_REF` to build another
branch or tag. The script wraps `cargo xtask package --format arch`; the build is unverified
on each Arch derivative.
