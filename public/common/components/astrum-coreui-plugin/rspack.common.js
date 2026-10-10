
const { rspack } = require('@rspack/core');
const path = require('path');
const fs = require('fs');
const { execSync } = require('child_process');

const swcOptions = JSON.parse(fs.readFileSync(path.resolve(__dirname, '.swcrc')), 'utf-8');

let gitCommit = 'unknown';
try {
  gitCommit = execSync('git rev-parse --short=7 HEAD', { encoding: 'utf-8' }).trim();
} catch (_) { /* not in git */ }

module.exports = {
  entry: {
    'astrum-coreui-bundle': './src/frontend/src/index.js'
  },
  module: {
    rules: [
      {
        test: /\.[jt]sx?$/,
        exclude: /node_modules/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: swcOptions
          }
        ]
      },
      {
        test: /\.js$/,
        include: /node_modules[\/\\]fuse\.js/,
        use: [
          {
            loader: 'builtin:swc-loader'
          }
        ]
      },
      {
        test: /\.s?css$/,
        oneOf: [
          {
            // @fontsource packages ship plain CSS whose @font-face rules reference their
            // font files with relative url(./files/*.woff2) paths. Those url()s must be
            // resolved so the referenced files are handed to the asset/resource rule below
            // and actually emitted. Scoped to node_modules/@fontsource only, so all
            // application and react-shared-components CSS keeps the url: false behaviour
            // it relies on. No sass-loader: these are plain .css files.
            include: /node_modules[\/\\]@fontsource[\/\\]/,
            use: [
              {
                loader: rspack.CssExtractRspackPlugin.loader
              },
              {
                loader: 'css-loader'
              }
            ]
          },
          {
            use: [
              {
                loader: rspack.CssExtractRspackPlugin.loader
              },
              {
                loader: 'css-loader',
                options: { url: false } // disable build-tile resolution of url() paths
              },
              {
                loader: 'sass-loader'
              }
            ]
          }
        ]
      },
      {
        test: /\.(png)$/,
        type: 'asset',
        generator: {
          filename: 'img/[name].[contenthash:8][ext]'
        }
      },
      {
        test: /\.(ttf|eot|woff2?|svg)$/,
        type: 'asset/resource',
        generator: {
          filename: 'fonts/[name].[contenthash:8][ext]'
        }
      }
    ]
  },
  plugins: [
    new rspack.CssExtractRspackPlugin({
      filename: '[name].css',
      chunkFilename: '[name].[contenthash:8].css',
      // Suppress CSS chunk ordering warnings. These arise because astrum-ui-plugin
      // SCSS files are imported by many lazy-loaded chunks via different code paths,
      // so rspack cannot guarantee a consistent load order across chunks. The
      // warnings are harmless: our component styles are self-contained and do not
      // depend on a specific inter-file load order.
      ignoreOrder: true
    }),
    new rspack.DefinePlugin({
      __AS_BUILD_COMMIT__: JSON.stringify(gitCommit),
      // Set GLOBALART_INTERNAL=true in your environment to include internal test pages.
      // These are excluded from production customer builds when this is not set.
      // Usage: GLOBALART_INTERNAL=true yarn build-all
      __GLOBALART_INTERNAL__: JSON.stringify(process.env.GLOBALART_INTERNAL === 'true'),
    }),
    new rspack.CopyRspackPlugin({
      patterns: [{
        from: path.resolve(__dirname, '../../../../node_modules/@globalart/react-shared-components/assets/'),
        to: path.resolve(__dirname, 'target/classes/assets')
      }]
    })
  ],
  resolve: {
    extensions: ['.js', '.jsx', '.ts', '.tsx'],
    alias: {
      '@': path.resolve(__dirname, './src/frontend/src'),
      '@nosc': path.resolve(__dirname, './src/frontend/src/nosc'),
      // Force @radix-ui/themes to always use its ESM build so that coreui's
      // own TypeScript (uses import → ESM) and astrum-ui-plugin's Babel-compiled
      // CJS (uses require → CJS) resolve to the SAME file, preventing duplicate
      // React.createContext() calls that break Theme context providers.
      '@radix-ui/themes': path.resolve(__dirname, '../../../../node_modules/@radix-ui/themes/dist/esm/index.js'),
    }
  },
  optimization: {
    splitChunks: false
  }
};
