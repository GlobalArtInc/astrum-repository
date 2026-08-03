

// Jest stub for the 'fflate' zip/compression library used by FileUploadZone.tsx
// to inspect uploaded zip archives. fflate is not listed as a package.json
// dependency, so Jest cannot resolve it; this stub satisfies the moduleNameMapper
// entry in jest.config.js so tests can run. Individual tests that exercise zip
// behaviour override these mocks via jest.mock('fflate', ...).
module.exports = {
  unzipSync: jest.fn(() => ({})),
  strFromU8: jest.fn((buf) => new TextDecoder().decode(buf)),
};
