module.exports = api => ({
  presets: [
    [
      '@babel/preset-env',
      {
        targets: api.env('test') ? { node: 'current' } : ['>0.1%', 'not dead'],
        modules: api.env('test') && 'commonjs',
      },
    ],
    ['@babel/preset-react', { development: api.env(['development', 'test']) }],
  ],
  plugins: [
    ['@babel/plugin-transform-runtime', { helpers: true }],
    '@babel/plugin-proposal-class-properties',
  ],
});
