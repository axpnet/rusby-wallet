#!/bin/bash
set -e

echo "Building Rusby Wallet Extension..."

# 0. Build WalletConnect bundle
echo ">> Building WalletConnect bundle..."
if [ -f "walletconnect/package.json" ]; then
    cd walletconnect
    npm ci --prefer-offline 2>/dev/null || npm install
    npm run build
    cd ..
fi

# 1. Build WASM with trunk (release mode, public-url relative)
echo ">> Building WASM..."
trunk build --release --public-url "./"

# 2. Prepare extension directory
EXT_DIR="extension-dist"
rm -rf "$EXT_DIR"
mkdir -p "$EXT_DIR/icons"

# 3. Copy trunk output (dist/) into extension
cp -r dist/* "$EXT_DIR/"

# 4. Copy manifest.json
cp extension/manifest.json "$EXT_DIR/"

# 4b. Copy extension JS files (background, content-script, inpage)
echo ">> Copying extension scripts..."
cp extension/background.js "$EXT_DIR/"
cp extension/content-script.js "$EXT_DIR/"
cp extension/inpage.js "$EXT_DIR/"
if [ -f "extension/wc-bundle.js" ]; then
    cp extension/wc-bundle.js "$EXT_DIR/"
fi

# 4c. Fix inline script → external file for CSP compliance
echo ">> Fixing inline script for extension CSP..."
# Extract JS and WASM filenames
JS_FILE=$(ls "$EXT_DIR"/wallet-ui-*.js 2>/dev/null | head -1 | xargs basename)
WASM_FILE=$(ls "$EXT_DIR"/wallet-ui-*_bg.wasm 2>/dev/null | head -1 | xargs basename)

if [ -n "$JS_FILE" ] && [ -n "$WASM_FILE" ]; then
    # Create external bootstrap script that works in extension context
    cat > "$EXT_DIR/bootstrap.js" << 'JSEOF'
try {
    const jsURL = (typeof chrome !== 'undefined' && chrome.runtime && chrome.runtime.getURL)
        ? chrome.runtime.getURL('JS_FILE_PLACEHOLDER')
        : './JS_FILE_PLACEHOLDER';
    const wasmURL = (typeof chrome !== 'undefined' && chrome.runtime && chrome.runtime.getURL)
        ? chrome.runtime.getURL('WASM_FILE_PLACEHOLDER')
        : './WASM_FILE_PLACEHOLDER';

    const mod = await import(jsURL);
    const init = mod.default;

    // Compile WASM from ArrayBuffer to avoid fetch credential issues
    const wasmResp = await fetch(wasmURL);
    const wasmBytes = await wasmResp.arrayBuffer();
    const wasmModule = await WebAssembly.compile(wasmBytes);

    await init({ module_or_path: wasmModule });
} catch (e) {
    console.error('Rusby bootstrap error:', e);
    document.body.innerHTML = '<pre style="color:red;padding:20px;">Error: ' + e.message + '</pre>';
}
JSEOF
    # Replace placeholders with actual filenames
    sed -i "s/JS_FILE_PLACEHOLDER/${JS_FILE}/g; s/WASM_FILE_PLACEHOLDER/${WASM_FILE}/g" "$EXT_DIR/bootstrap.js"

    # Rewrite index.html: remove inline script, preload links, fix paths
    python3 -c "
import re
with open('${EXT_DIR}/index.html', 'r') as f:
    html = f.read()
# Remove inline script block
html = re.sub(r'<script type=\"module\">.*?</script>', '', html, flags=re.DOTALL)
# Remove preload/modulepreload links (cause credential mismatch in extensions)
html = re.sub(r'<link rel=\"modulepreload\"[^>]*>', '', html)
html = re.sub(r'<link rel=\"preload\"[^>]*>', '', html)
# Add external script before </body>
html = html.replace('</body>', '<script type=\"module\" src=\"bootstrap.js\"></script>\n</body>')
# Fix any remaining absolute paths to relative
html = html.replace('href=\"/', 'href=\"./')
html = html.replace('src=\"/', 'src=\"./')
# Remove integrity attributes
html = re.sub(r'\s+integrity=\"[^\"]*\"', '', html)
# Remove crossorigin attributes
html = re.sub(r'\s+crossorigin=\"[^\"]*\"', '', html)
# Inject extension CSP meta tag into <head>
csp = '<meta http-equiv=\"Content-Security-Policy\" content=\"default-src \\'self\\'; script-src \\'self\\' \\'wasm-unsafe-eval\\'; connect-src https:; style-src \\'self\\' \\'unsafe-inline\\'; img-src \\'self\\' data: https://nft-cdn.alchemy.com https://res.cloudinary.com https://ipfs.io https://gateway.pinata.cloud https://arweave.net https://img-cdn.magiceden.dev https://*.nftstorage.link;\" />'
html = html.replace('<head>', '<head>\n    ' + csp, 1)
with open('${EXT_DIR}/index.html', 'w') as f:
    f.write(html)
"
    echo ">> Bootstrap script created: bootstrap.js"
fi

# 5. Copy icons (if they exist)
if [ -d "extension/icons" ]; then
    cp extension/icons/* "$EXT_DIR/icons/" 2>/dev/null || true
fi

# 6. Generate placeholder icons if none exist
if [ ! -f "$EXT_DIR/icons/icon48.png" ]; then
    echo ">> No icons found, generating placeholders..."
    # Create simple SVG icons as placeholder
    for size in 16 48 128; do
        cat > "$EXT_DIR/icons/icon${size}.svg" << SVGEOF
<svg xmlns="http://www.w3.org/2000/svg" width="$size" height="$size" viewBox="0 0 $size $size">
  <rect width="$size" height="$size" rx="$(($size/5))" fill="#6c5ce7"/>
  <text x="50%" y="55%" text-anchor="middle" dominant-baseline="middle" fill="white" font-family="sans-serif" font-weight="bold" font-size="$(($size*4/10))">W</text>
</svg>
SVGEOF
    done
    # Update manifest to use SVGs
    sed -i 's/icon16\.png/icon16.svg/g; s/icon48\.png/icon48.svg/g; s/icon128\.png/icon128.svg/g' "$EXT_DIR/manifest.json"
fi

echo ""
echo "Extension built successfully in: $EXT_DIR/"
echo "Load it in Chrome: chrome://extensions -> Load unpacked -> select $EXT_DIR"
