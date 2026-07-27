
const isTest = process.env.NODE_ENV === 'test';

module.exports = {
  presets: [
    '@babel/preset-react',
    [
      '@babel/preset-env',
      {
        // IE 11 dropped per ASTRUM-52389; targets match published system requirements.
        targets: 'last 1 Chrome version, last 1 Firefox version, Firefox ESR, last 1 Safari version, last 1 Edge version',
        modules: isTest ? 'auto' : false
      }
    ],
    '@babel/preset-typescript'
  ],
  plugins: [
    '@babel/plugin-transform-class-properties',
    '@babel/plugin-transform-optional-chaining',
    '@babel/plugin-transform-runtime'
  ]
};
