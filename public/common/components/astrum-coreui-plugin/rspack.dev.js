
const {merge} = require('webpack-merge');
const path = require('path');

const common = require('./rspack.common');

module.exports = merge(common, {
  mode: 'development',
  devtool: 'eval-source-map',

  output: {
    filename: '[name].debug.js',
    chunkFilename: '[name].debug.js',
    path: path.resolve(__dirname, 'target', 'classes', 'static')
  },
  optimization: {
    chunkIds: 'named'
  }
});
