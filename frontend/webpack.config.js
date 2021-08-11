const path = require('path');
const webpack = require('webpack');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const CssMinimizerPlugin = require('css-minimizer-webpack-plugin');
const TerserWebpackPlugin = require('terser-webpack-plugin');
const ESLintWebpackPlugin = require('eslint-webpack-plugin');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');

const serverConfig = require('../config.json');

/**
  @returns {webpack.Configuration}
*/
module.exports = (_env, { mode }) => ({
  mode,
  devtool: 'source-map',
  entry: path.resolve(__dirname, 'src'),
  resolve: {
    extensions: ['.jsx', '.js', '.json'],
  },
  output: {
    path: path.resolve(__dirname, '..', serverConfig.server.static_files.path),
    publicPath: serverConfig.server.static_files.base_url,
    filename: 'assets/[name].[chunkhash:8].js',
    chunkFilename: 'assets/[name].[chunkhash:8].chunk.js',
    assetModuleFilename: 'assets/[name].[hash:8][ext]',
  },
  module: {
    parser: { javascript: { strictExportPresence: true } },
    rules: [
      {
        oneOf: [
          {
            test: /\.(jsx?|mjs)$/,
            include: path.resolve(__dirname, 'src'),
            loader: 'babel-loader',
            options: { cacheDirectory: true },
          },
          {
            test: /\.(s[ca]|c)ss$/,
            use: [
              { loader: MiniCssExtractPlugin.loader },
              { loader: 'css-loader', options: { importLoaders: 1 } },
              { loader: 'sass-loader' },
            ],
          },
          {
            exclude: [/\.(jsx?|mjs|html|json)$/, /^$/],
            type: 'asset/resource',
          },
        ],
      },
    ],
  },
  plugins: [
    new webpack.ProgressPlugin(),
    new CleanWebpackPlugin(),
    new webpack.DefinePlugin({
      'process.env': {
        CANVAS_WIDTH: JSON.stringify(serverConfig.canvas.width),
        CANVAS_HEIGHT: JSON.stringify(serverConfig.canvas.height),
      },
    }),
    new webpack.ProvidePlugin({
      Buffer: ['buffer', 'Buffer'],
    }),
    new ESLintWebpackPlugin({
      files: path.resolve(__dirname, 'src'),
      extensions: ['js', 'jsx'],
    }),
    new HtmlWebpackPlugin({
      template: path.resolve(__dirname, 'src', 'index.html'),
    }),
    new MiniCssExtractPlugin({
      filename: 'assets/[name].[contenthash:8].css',
      chunkFilename: 'assets/[name].[contenthash:8].chunk.css',
    }),
  ],
  optimization: {
    minimizer: [new TerserWebpackPlugin(), new CssMinimizerPlugin()],
  },
  devServer: {
    devMiddleware: {
      publicPath: serverConfig.server.static_files.base_url,
      index: serverConfig.server.static_files.index_file,
    },
    proxy: {
      '/api': {
        target: `http://${serverConfig.server.address}`,
        ws: true,
      },
    },
    client: {
      overlay: {
        errors: true,
        warnings: false,
      },
    },
  },
});
