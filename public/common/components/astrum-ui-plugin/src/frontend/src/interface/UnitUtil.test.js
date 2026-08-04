
import UnitUtil from './UnitUtil';

describe('UnitUtil', () => {
  describe('bytesToMegaBytes', () => {
    it('returns null for null input', () => {
      expect(UnitUtil.bytesToMegaBytes(null))
      .toBeNull();
    });
    it('returns undefined for undefined inout', () => {
      expect(UnitUtil.bytesToMegaBytes(undefined))
      .toBeUndefined();
    });
    it('returns empty string for empty string input', () => {
      expect(UnitUtil.bytesToMegaBytes(''))
      .toBe('');
    });
    it('returns 0 for 0 input', () => {
      expect(UnitUtil.bytesToMegaBytes(0))
      .toBe(0);
    });
    it('returns 1 megabyte for 1048576 bytes', () => {
      expect(UnitUtil.bytesToMegaBytes(1048576))
      .toBe(1);
    });
    // 1 Terabyte = 1048576 Megabytes
    it('returns 1048576(1 Terabyte) megabytes for 1099511627776 bytes', () => {
      expect(UnitUtil.bytesToMegaBytes(1099511627776))
      .toBe(1048576);
    });
    it('returns 1048576(1 Terabyte) megabytes for string input 1099511627776 bytes', () => {
      expect(UnitUtil.bytesToMegaBytes('1099511627776'))
      .toBe(1048576);
    });
  });
  describe('megaBytesToBytes', () => {
    it('returns null for null input', () => {
      expect(UnitUtil.megaBytesToBytes(null))
      .toBeNull();
    });
    it('returns undefined for undefined inout', () => {
      expect(UnitUtil.megaBytesToBytes(undefined))
      .toBeUndefined();
    });
    it('returns empty string for empty string input', () => {
      expect(UnitUtil.megaBytesToBytes(''))
      .toBe('');
    });
    it('returns 0 for 0 input', () => {
      expect(UnitUtil.megaBytesToBytes(0))
      .toBe(0);
    });
    it('returns 1048576 bytes for 1 megabyte', () => {
      expect(UnitUtil.megaBytesToBytes(1))
      .toBe(1048576);
    });
    // 1 Terabyte = 1048576 Megabytes
    it('returns 1099511627776(1 Terabyte) for 1048576 megabyte', () => {
      expect(UnitUtil.megaBytesToBytes(1048576))
      .toBe(1099511627776);
    });
    it('returns 1099511627776(1 Terabyte) for string input 1048576 megabyte', () => {
      expect(UnitUtil.megaBytesToBytes('1048576'))
      .toBe(1099511627776);
    });
  });
});
