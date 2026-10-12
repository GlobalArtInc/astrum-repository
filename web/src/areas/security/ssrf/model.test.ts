import { describe, expect, it } from 'vitest';
import { normalizeDomain, validDomain, validNetwork } from './model';

describe('SSRF list validation', () => {
  it('accepts addresses and networks the server accepts', () => {
    for (const value of [
      '127.0.0.1',
      '10.0.0.0/8',
      '0.0.0.0/0',
      '192.168.1.100/32',
      '::1',
      'fd00::/8',
      '::ffff:10.0.0.1',
    ]) {
      expect(validNetwork(value), value).toBe(true);
    }
  });

  it('refuses what is not an address or has a bad prefix', () => {
    for (const value of [
      '10.0.0',
      '256.1.1.1',
      '010.0.0.1',
      '10.0.0.0/33',
      '10.0.0.0/',
      'fd00::/129',
      'localhost',
      '[::1]',
      'fe80::1%eth0',
      '1.2.3.4/x',
    ]) {
      expect(validNetwork(value), value).toBe(false);
    }
  });

  it('accepts host names and wildcard domains', () => {
    for (const value of [
      'registry.local',
      '*.corp.example',
      'Example.ORG.',
      'my_host',
      'xn--e1afmkfd.xn--p1ai',
      'пример.рф',
    ]) {
      expect(validDomain(value), value).toBe(true);
    }
    for (const value of [
      '',
      '*.',
      '-bad.example',
      'bad-.example',
      'a..b',
      'with space.example',
      '*.*.example',
      'http://x',
    ]) {
      expect(validDomain(value), value).toBe(false);
    }
    expect(normalizeDomain('  Registry.Local ')).toBe('registry.local');
  });
});
