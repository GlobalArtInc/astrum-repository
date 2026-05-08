
const { rspack } = require('@rspack/core');
const path = require('path');
const fs = require('fs');

const swcOptions = JSON.parse(fs.readFileSync(path.resolve(__dirname, '.swcrc')), 'utf-8');

module.exports = {
  entry: {
    'astrum-rapture-bundle': './src/frontend/src/index.js'
  },
  module: {
    rules: [
      {
        test: /\.jsx?$/,
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
      filename: '[name].css'
    }),
    new rspack.CopyRspackPlugin({
      patterns: [{
        from: path.resolve(__dirname, '../../../../node_modules/@globalart/react-shared-components/assets/'),
        to: path.resolve(__dirname, 'target/classes/assets')
      }]
    })
  ],
  resolve: {
    extensions: ['.js', '.jsx']
  }
};
