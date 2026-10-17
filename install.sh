#!/bin/sh
# tubevibe installer for Linux and macOS. Puts the latest release's program
# in ~/.local/bin (or $TUBEVIBE_BIN_DIR); where there is no ready-made one
# (or with TUBEVIBE_FROM_SOURCE=1) it builds it with cargo instead, checking
# the compiler and Rust first. Either way it checks mpv and yt-dlp and prints
# the commands for whatever is missing.
#
#   curl -sSf https://raw.githubusercontent.com/mortbrew/tubevibe/main/install.sh | sh

set -u

REPO=mortbrew/tubevibe
BIN_DIR=${TUBEVIBE_BIN_DIR:-$HOME/.local/bin}

RUST_MIN=1.88
# yt-dlp older than this many days often fails: YouTube changes all the time.
YT_DLP_MAX_AGE_DAYS=90

case "${LANG:-}${LC_ALL:-}" in
    *es_* | es*) es=1 ;;
    *) es=0 ;;
esac
say() { if [ "$es" = 1 ]; then printf '%s\n' "$1"; else printf '%s\n' "$2"; fi; }
has() { command -v "$1" >/dev/null 2>&1; }

# Package manager, as src/doctor.rs picks it.
manager=unknown
if [ "$(uname -s)" = Darwin ]; then
    manager=brew
elif [ -r /etc/os-release ]; then
    ids=$(. /etc/os-release; echo "${ID:-} ${ID_LIKE:-}")
    for id in $ids; do
        case "$id" in
            debian | ubuntu) manager=apt ;;
            arch | manjaro | endeavouros) manager=pacman ;;
            fedora | rhel | centos) manager=dnf ;;
            opensuse* | suse) manager=zypper ;;
            *) continue ;;
        esac
        break
    done
fi

install_cmd() { # install_cmd <package>
    case "$manager" in
        apt) echo "sudo apt install -y $1" ;;
        pacman) echo "sudo pacman -S --needed $1" ;;
        dnf) echo "sudo dnf install -y $1" ;;
        zypper) echo "sudo zypper install -y $1" ;;
        brew) echo "brew install $1" ;;
        *) echo "# $(say "instalá $1 con el gestor de paquetes de tu sistema" "install $1 with your system's package manager")" ;;
    esac
}

# 0. The ready-made program from the latest release, when there is one for
#    this machine (file names as .github/workflows/release.yml makes them).
case "$(uname -s)-$(uname -m)" in
    Linux-x86_64 | Linux-amd64) asset=tubevibe-x86_64-linux.tar.gz ;;
    Linux-aarch64 | Linux-arm64) asset=tubevibe-aarch64-linux.tar.gz ;;
    Darwin-arm64) asset=tubevibe-aarch64-macos.tar.gz ;;
    *) asset="" ;;
esac
downloaded=0
if [ -n "$asset" ] && [ "${TUBEVIBE_FROM_SOURCE:-0}" != 1 ] && has curl && has tar; then
    say "Bajando tubevibe ($asset)…" "Downloading tubevibe ($asset)…"
    tmp=$(mktemp -d)
    if curl -fsL "https://github.com/$REPO/releases/latest/download/$asset" -o "$tmp/tubevibe.tar.gz" \
        && tar -xzf "$tmp/tubevibe.tar.gz" -C "$tmp" && [ -f "$tmp/tubevibe/tubevibe" ] \
        && mkdir -p "$BIN_DIR" && cp "$tmp/tubevibe/tubevibe" "$BIN_DIR/tubevibe.new" \
        && chmod 755 "$BIN_DIR/tubevibe.new" && mv -f "$BIN_DIR/tubevibe.new" "$BIN_DIR/tubevibe"; then
        downloaded=1
        say "Instalado en $BIN_DIR/tubevibe" "Installed at $BIN_DIR/tubevibe"
    else
        say "No hay una versión publicada para bajar: lo compilo." "No published release to download: building it instead."
    fi
    rm -rf "$tmp"
fi

# Missing pieces: one line each for the report, and the commands that fix them.
build_missing="" build_fix=""
run_missing="" run_fix=""
add_build() { build_missing="$build_missing  - $1
"; build_fix="$build_fix$2
"; }
add_run() { run_missing="$run_missing  - $1
"; run_fix="$run_fix$2
"; }

# 1. A C compiler (the TLS crate `ring` builds C code); only to build it.
if [ "$downloaded" = 0 ] && ! has cc && ! has gcc && ! has clang; then
    case "$manager" in
        apt) fix=$(install_cmd build-essential) ;;
        pacman) fix=$(install_cmd base-devel) ;;
        brew) fix="xcode-select --install" ;;
        *) fix=$(install_cmd gcc) ;;
    esac
    add_build "$(say "compilador de C (cc)" "C compiler (cc)")" "$fix"
fi

# 2. Rust >= RUST_MIN. Distro packages (Ubuntu 22.04 ships 1.75) are too old:
#    they stop at "feature `edition2024` is required".
user_path=$PATH
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
version_ge() { # version_ge 1.75.0 1.88
    [ "$(printf '%s\n%s\n' "$2" "$1" | sort -V | head -n1)" = "$2" ]
}
rustup_install="curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. \"\$HOME/.cargo/env\""
if [ "$downloaded" = 1 ]; then
    :
elif ! has cargo; then
    add_build "Rust $RUST_MIN+ ($(say "no está instalado" "not installed"))" "$rustup_install"
else
    rust=$(cargo --version 2>/dev/null | awk '{print $2}')
    if ! version_ge "${rust:-0}" "$RUST_MIN"; then
        if has rustup; then
            fix="rustup update stable && rustup default stable"
        else
            # rustup goes to ~/.cargo/bin, ahead of the distro's /usr/bin/cargo.
            fix=$rustup_install
        fi
        add_build "Rust $rust → $(say "hace falta" "needs") $RUST_MIN+ ($(command -v cargo))" "$fix"
    fi
fi

# 3. What tubevibe runs: mpv and a current yt-dlp.
if ! has mpv; then
    add_run "mpv" "$(install_cmd mpv)"
fi
yt_dlp_fresh() {
    case "$manager" in
        apt) echo "sudo apt install -y pipx && pipx install yt-dlp && pipx ensurepath" ;;
        brew) echo "brew install yt-dlp" ;;
        *) install_cmd yt-dlp ;;
    esac
}
if ! has yt-dlp; then
    add_run "yt-dlp" "$(yt_dlp_fresh)"
else
    path=$(command -v yt-dlp)
    v=$(yt-dlp --version 2>/dev/null | head -n1)
    age=$(echo "$(echo "$v" | cut -d. -f1-3) $(date +'%Y %m %d')" | awk -F'[ .]' '{print ($4*365+($5-1)*30+$6) - ($1*365+($2-1)*30+$3)}')
    if [ "${age:-0}" -gt "$YT_DLP_MAX_AGE_DAYS" ]; then
        case "$path" in
            *pipx* | */.local/bin/*) fix="pipx upgrade yt-dlp" ;;
            *)
                case "$manager" in
                    # The distro's yt-dlp lags months behind; swap it for pipx's.
                    apt) fix="sudo apt remove -y yt-dlp && $(yt_dlp_fresh)" ;;
                    pacman) fix="sudo pacman -Syu yt-dlp" ;;
                    dnf) fix="sudo dnf upgrade -y yt-dlp" ;;
                    zypper) fix="sudo zypper update -y yt-dlp" ;;
                    brew) fix="brew upgrade yt-dlp" ;;
                    *) fix="yt-dlp -U" ;;
                esac
                ;;
        esac
        add_run "yt-dlp $v ($(say "viejo, YouTube lo rompe" "too old, YouTube breaks it")) $path" "$fix"
    fi
fi

report() { # report <missing> <commands>
    printf '%s\n' "$1"
    say "Comandos (copiá y pegá):" "Commands (copy and paste):"
    echo
    printf '%s\n' "$2"
}

if [ -n "$build_missing" ]; then
    say "Para compilar tubevibe falta:" "To build tubevibe you need:"
    report "$build_missing" "$build_fix"
    if [ -n "$run_missing" ]; then
        say "Y para usarlo:" "And to use it:"
        report "$run_missing" "$run_fix"
    fi
    say "Después volvé a correr este instalador." "Then run this installer again."
    exit 1
fi

if [ "$downloaded" = 0 ]; then
    say "Compilando tubevibe (unos minutos)…" "Building tubevibe (a few minutes)…"
    cargo install --locked --git "https://github.com/$REPO" || exit 1
    bin_dir=$HOME/.cargo/bin
else
    bin_dir=$BIN_DIR
fi

if [ -n "$run_missing" ]; then
    echo
    say "tubevibe está instalado, pero para usarlo falta:" "tubevibe is installed, but to use it you need:"
    report "$run_missing" "$run_fix"
fi
case ":$user_path:" in
    *":$bin_dir:"*) say "Listo: ejecutá tubevibe" "Done: run tubevibe" ;;
    *) if [ "$downloaded" = 1 ]; then
        say "$bin_dir no está en tu PATH: agregalo (export PATH=\"$bin_dir:\$PATH\" en ~/.profile) o ejecutá: $bin_dir/tubevibe" \
            "$bin_dir is not in your PATH: add it (export PATH=\"$bin_dir:\$PATH\" in ~/.profile) or run: $bin_dir/tubevibe"
    else
        say "Abrí una terminal nueva (o corré: . \"\$HOME/.cargo/env\") y ejecutá: tubevibe" \
            "Open a new terminal (or run: . \"\$HOME/.cargo/env\") and run: tubevibe"
    fi ;;
esac
