#!/usr/bin/env bash
set -euo pipefail

version="${1:?version required}"
arch="${2:?architecture required}"
root="$(pwd)"
prefix="bibillmproxy-${version}-linux-${arch}"
mkdir -p dist

portable="dist/${prefix}"
mkdir -p "${portable}/docs" "${portable}/assets"
install -m 755 target/release/lite-gui target/release/lite "${portable}/"
install -m 644 LICENSE README.md config.yaml.example "${portable}/"
install -m 644 docs/desktop-gui.md "${portable}/docs/"
install -m 644 assets/bibillmproxy-icon.png "${portable}/assets/"
tar -czf "dist/${prefix}.tar.gz" -C dist "${prefix}"

desktop="dist/bibillmproxy.desktop"
cat > "${desktop}" <<'EOF'
[Desktop Entry]
Name=BibiLLMProxy
Exec=lite-gui
Icon=bibillmproxy
Type=Application
Categories=Network;Development;
Terminal=false
EOF

appdir="dist/BibiLLMProxy.AppDir"
mkdir -p "${appdir}/usr/bin" "${appdir}/usr/share/applications" "${appdir}/usr/share/icons/hicolor/512x512/apps"
install -m 755 target/release/lite-gui target/release/lite "${appdir}/usr/bin/"
install -m 644 "${desktop}" "${appdir}/bibillmproxy.desktop"
install -m 644 "${desktop}" "${appdir}/usr/share/applications/"
install -m 644 assets/bibillmproxy-icon.png "${appdir}/bibillmproxy.png"
install -m 644 assets/bibillmproxy-icon.png "${appdir}/usr/share/icons/hicolor/512x512/apps/bibillmproxy.png"
cat > "${appdir}/AppRun" <<'EOF'
#!/bin/sh
APPDIR="$(dirname "$(readlink -f "$0")")"
exec "$APPDIR/usr/bin/lite-gui" "$@"
EOF
chmod +x "${appdir}/AppRun"
curl -fsSL "https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-${arch}.AppImage" -o dist/appimagetool.AppImage
chmod +x dist/appimagetool.AppImage
cd dist
./appimagetool.AppImage --appimage-extract >/dev/null
ARCH="${arch}" ./squashfs-root/AppRun BibiLLMProxy.AppDir "${prefix}.AppImage"
cd "${root}"

rpmdir="dist/rpmbuild"
mkdir -p "${rpmdir}/BUILD" "${rpmdir}/RPMS" "${rpmdir}/SOURCES" "${rpmdir}/SPECS" "${rpmdir}/SRPMS"
cat > "${rpmdir}/SPECS/bibillmproxy.spec" <<EOF
Name: bibillmproxy
Version: ${version}
Release: 1
Summary: LiteLLM-compatible desktop proxy console
License: MIT
URL: https://github.com/bibiparrot/bibillmproxy

%description
Desktop console and local API gateway for OpenAI Responses and Anthropic Messages.

%install
install -d %{buildroot}/usr/bin %{buildroot}/usr/share/applications %{buildroot}/usr/share/icons/hicolor/512x512/apps %{buildroot}/usr/share/doc/bibillmproxy
install -m 755 "${root}/target/release/lite-gui" "${root}/target/release/lite" %{buildroot}/usr/bin/
install -m 644 "${root}/dist/bibillmproxy.desktop" %{buildroot}/usr/share/applications/
install -m 644 "${root}/assets/bibillmproxy-icon.png" %{buildroot}/usr/share/icons/hicolor/512x512/apps/bibillmproxy.png
install -m 644 "${root}/LICENSE" "${root}/README.md" "${root}/docs/desktop-gui.md" %{buildroot}/usr/share/doc/bibillmproxy/

%files
/usr/bin/lite-gui
/usr/bin/lite
/usr/share/applications/bibillmproxy.desktop
/usr/share/icons/hicolor/512x512/apps/bibillmproxy.png
/usr/share/doc/bibillmproxy/*
EOF
rpmbuild --define "_topdir ${root}/${rpmdir}" -bb "${rpmdir}/SPECS/bibillmproxy.spec"
find "${rpmdir}/RPMS" -name '*.rpm' -exec cp {} "dist/${prefix}.rpm" \;
test -s "dist/${prefix}.rpm"
