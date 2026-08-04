
import { getVersionMajorMinor } from './versionUtil';

describe('versionUtil', function() {
  describe('getVersionMajorMinor', function() {
    it('returns the input if passed null or undefined', function() {
      expect(getVersionMajorMinor(null)).toBe(null);
      expect(getVersionMajorMinor(undefined)).toBe(undefined);
    });

    it('returns the empty string if passed the empty string', function() {
      expect(getVersionMajorMinor('')).toBe('');
    });

    it('returns the entire input string if the input does not contain a "." character', function() {
      expect(getVersionMajorMinor('asdof#adfovn(aklndkjn,adionaoiqweqqnf894h-+!@#$%^&*(<>/?|\\\'":;…'))
          .toBe('asdof#adfovn(aklndkjn,adionaoiqweqqnf894h-+!@#$%^&*(<>/?|\\\'":;…');
    });

    it('returns the entire input string if the string contains exactly one "." character', function() {
      expect(getVersionMajorMinor('asdof#adfovn(aklndkjn,ad.ionaoiqweqqnf894h-+!@#$%^&*(<>/?|\\\'":;…'))
          .toBe('asdof#adfovn(aklndkjn,ad.ionaoiqweqqnf894h-+!@#$%^&*(<>/?|\\\'":;…');
    });

    it('returns the prefix of the input string up to but not including the second "." character', function() {
      expect(getVersionMajorMinor('1.2.')).toBe('1.2');
      expect(getVersionMajorMinor('1.2.3')).toBe('1.2');
      expect(getVersionMajorMinor('1-a.2-b.3-c')).toBe('1-a.2-b');
      expect(getVersionMajorMinor('1.2.3-SNAPSHOT')).toBe('1.2');
      expect(getVersionMajorMinor('aa.bb.cc-SNAPSHOT')).toBe('aa.bb');
      expect(getVersionMajorMinor('aa..cc-SNAPSHOT')).toBe('aa.');
    });
  });
});
