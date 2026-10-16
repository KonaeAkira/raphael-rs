<!-- runtime-requirements -->
## Runtime requirements

The native app renders its interface in the system webview. The following must be installed on the machine that runs it:

- **Windows:** the [WebView2 runtime](https://developer.microsoft.com/microsoft-edge/webview2/). It ships with Windows 11 and with up-to-date Windows 10, so no action is usually needed.
- **Linux:** WebKitGTK 4.1, GTK 3 and libxdo.
  - Arch: `pacman -S webkit2gtk-4.1 gtk3 xdotool`
  - Debian/Ubuntu: `apt install libwebkit2gtk-4.1-0 libgtk-3-0 libxdo3`
  - Fedora: `dnf install webkit2gtk4.1 gtk3 xdotool`

## Updating from 0.28 or earlier

Versions before 0.29 cannot update themselves to this release. Download `Raphael_x86_64_windows.exe` or `Raphael_x86_64_linux` from the assets below and replace the old executable. Your settings and saved rotations are kept.
