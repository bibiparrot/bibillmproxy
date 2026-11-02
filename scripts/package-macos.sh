#!/usr/bin/env bash
set -euo pipefail

version="${1:?version required}"
arch="${2:?architecture required}"
prefix="bibillmproxy-${version}-${arch}-macOS"
mkdir -p dist
app="dist/BibiLLMProxy.app"
mkdir -p "${app}/Contents/MacOS" "${app}/Contents/Resources"
install -m 755 target/release/lite-gui target/release/lite "${app}/Contents/MacOS/"
install -m 644 config.yaml.example LICENSE README.md "${app}/Contents/Resources/"
cat > "${app}/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>BibiLLMProxy</string>
<key>CFBundleDisplayName</key><string>BibiLLMProxy</string>
<key>CFBundleIdentifier</key><string>com.bibiparrot.bibillmproxy</string>
<key>CFBundleVersion</key><string>${version}</string>
<key>CFBundleShortVersionString</key><string>${version}</string>
<key>CFBundleExecutable</key><string>lite-gui</string>
<key>CFBundleIconFile</key><string>bibillmproxy.icns</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
EOF
iconset="dist/bibillmproxy.iconset"
mkdir -p "${iconset}"
for size in 16 32 128 256 512; do
  sips -s format png -z "${size}" "${size}" assets/bibillmproxy-icon.png --out "${iconset}/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  sips -s format png -z "${double}" "${double}" assets/bibillmproxy-icon.png --out "${iconset}/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "${iconset}" -o "${app}/Contents/Resources/bibillmproxy.icns"
ditto -c -k --sequesterRsrc --keepParent "${app}" "dist/${prefix}.zip"
mkdir -p dist/dmgroot
cp -R "${app}" dist/dmgroot/
ln -s /Applications dist/dmgroot/Applications
hdiutil create -volname BibiLLMProxy -srcfolder dist/dmgroot -ov -format UDZO "dist/${prefix}.dmg"
mkdir -p dist/pkgroot/Applications
cp -R "${app}" dist/pkgroot/Applications/
pkgbuild --root dist/pkgroot --identifier com.bibiparrot.bibillmproxy --version "${version}" --install-location / "dist/${prefix}.pkg"
