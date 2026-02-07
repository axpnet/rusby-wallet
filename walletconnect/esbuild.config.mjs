import { build } from 'esbuild';

await build({
  entryPoints: ['src/wc-bridge.js'],
  bundle: true,
  format: 'esm',
  outfile: '../extension/wc-bundle.js',
  platform: 'browser',
  target: 'chrome110',
  minify: true,
  define: {
    'global': 'globalThis',
    'process.env.NODE_ENV': '"production"',
  },
});

console.log('WalletConnect bundle built → extension/wc-bundle.js');
