const path = require('path');

const blocks = require('@webpack-blocks/webpack');

const assets = require('@webpack-blocks/assets');
const babel = require('@webpack-blocks/babel');
const devServer = require('@webpack-blocks/dev-server');
const eslint = require('@webpack-blocks/eslint');

const HtmlWebpackPlugin = require('html-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const OptimizeCSSAssetsPlugin = require('optimize-css-assets-webpack-plugin');
const TerserPlugin = require('terser-webpack-plugin');

const serverConfig = require('../config.json');

const NODE_ENV = process.env.NODE_ENV || 'development';

// constructs a URL (relative to the base URL of static files) of an asset from
// the given components
function assetPath(filename, hash, extension) {
  let fullPath = `assets/${filename}`;

  if (NODE_ENV === 'production') fullPath += `.${hash}`;
  fullPath += `.${extension}`;

  return fullPath;
}

// returns an absolute version of a path that is relative to this file
function resolve(...paths) {
  return path.resolve(__dirname, ...paths);
}

// regular expressions for common file types
const JS_FILES = /\.(jsx?|mjs)$/;
const STYLESHEET_FILES = /\.(s[ca]|c)ss$/;
const IMAGE_FILES = /\.(png|jpe?g|gif|bmp)$/;

// a simple webpack block that adds a loader
const addLoader = (loader, options) => (context, util) =>
  util.addLoader({ use: [{ loader, options }], ...context.match });

module.exports = blocks.createConfig([
  blocks.setMode(NODE_ENV),

  blocks.resolve({ extensions: ['.jsx'] }),
  blocks.entryPoint(resolve('src', 'index')),

  blocks.setOutput({
    filename: assetPath('[name]', '[chunkhash:8]', 'js'),
    chunkFilename: assetPath('[name]', '[chunkhash:8]', 'chunk.js'),
    publicPath: serverConfig.server.static_files.base_url,
  }),
  blocks.env('production', [
    blocks.setOutput({
      path: resolve('..', serverConfig.server.static_files.path),
    }),
  ]),

  blocks.match(JS_FILES, { include: resolve('src') }, [eslint(), babel()]),

  blocks.match(STYLESHEET_FILES, [
    addLoader(MiniCssExtractPlugin.loader),
    assets.css({
      sourceMap: true,
      importLoaders: 1,
      // disable style-loader because mini-css-extract-plugin is used instead
      styleLoader: false,
    }),
    // replace this with the official sass block when webpack-blocks 2.0 is released
    addLoader('sass-loader', { sourceMap: true }),
  ]),
  blocks.addPlugins([
    new MiniCssExtractPlugin({
      filename: assetPath('[name]', '[chunkhash:8]', 'css'),
      chunkFilename: assetPath('[name]', '[chunkhash:8]', 'chunk.css'),
    }),
  ]),

  blocks.match(IMAGE_FILES, [
    assets.url({
      limit: 8 * 1024, // 8 KiB
      name: assetPath('assets/[name]', '[hash:8]', '[ext]'),
    }),
  ]),

  blocks.match(
    undefined,
    { exclude: [JS_FILES, STYLESHEET_FILES, IMAGE_FILES, /\.(json|html)$/] },
    [assets.file({ name: assetPath('assets/[name]', '[hash:8]', '[ext]') })],
  ),

  blocks.setEnv({
    NODE_ENV,
    CANVAS_WIDTH: serverConfig.canvas.width,
    CANVAS_HEIGHT: serverConfig.canvas.height,
  }),

  blocks.addPlugins([
    new HtmlWebpackPlugin({
      inject: 'body',
      template: resolve('src', 'index.html'),
    }),
  ]),

  blocks.optimization({
    // extracts webpack runtime into a different chunk
    runtimeChunk: true,
    // https://gist.github.com/sokra/1522d586b8e5c0f5072d7565c2bee693
    splitChunks: { chunks: 'all' },
    // use file paths as module IDs
    // https://medium.com/webpack/predictable-long-term-caching-with-webpack-d3eee1d3fa31
    namedModules: true,
  }),

  blocks.setDevTool('source-map'),
  blocks.env('development', [
    devServer({
      // mimic real backend settings
      publicPath: serverConfig.server.static_files.base_url,
      index: serverConfig.server.static_files.index_file,
      // don't redirect to the index file on 404's
      historyApiFallback: false,
      // proxy API requests to the real backend
      proxy: {
        '/api': {
          target: `http://${serverConfig.server.address}`,
          ws: true, // enable WebSockets
        },
      },
      // show full-screen overlay in the browser on compiler errors or warnings
      overlay: true,
    }),
  ]),

  blocks.optimization({
    minimizer: [
      new TerserPlugin({
        cache: true,
        parallel: true,
        sourceMap: true,
      }),
      new OptimizeCSSAssetsPlugin(),
    ],
  }),
]);
