
const {merge} = require('webpack-merge');
const path = require('path');
const {rspack} = require('@rspack/core');

const common = require('./rspack.common');

module.exports = merge(common, {
  mode: 'production',
  devtool: 'source-map',

  output: {
    filename: '[name].js',
    path: path.resolve(__dirname, 'target', 'classes', 'static')
  },

  optimization: {
    minimize: true,
    minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin(),
      new rspack.LightningCssMinimizerRspackPlugin()
    ]
  }
});
