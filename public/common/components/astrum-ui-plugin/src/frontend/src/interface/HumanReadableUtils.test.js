
import HumanReadableUtils from './HumanReadableUtils';

describe('HumanReadableUtils', () => {
  describe('bytesToString', () => {
    it('returns "Unavailable" for a negative byte value', () => {
      expect(HumanReadableUtils.bytesToString(-1)).toBe('Unavailable');
    });

    it('formats small byte values correctly with default format (SI)', () => {
      expect(HumanReadableUtils.bytesToString(2)).toBe('2.00 Bytes');
    });

    it('formats kilobytes correctly with default format (SI)', () => {
      // 2000 bytes → 2.00 kB (SI)
      expect(HumanReadableUtils.bytesToString(2000)).toBe('2.00 kB');
    });

    it('formats megabytes correctly with default format (SI)', () => {
      // 1048576 bytes → 1.05 MB (SI)
      expect(HumanReadableUtils.bytesToString(1048576)).toBe('1.05 MB');
    });

    it('formats large values correctly with default format (SI)', () => {
      // 2 GB in bytes → 2,147,483,648 bytes
      expect(HumanReadableUtils.bytesToString(2147483648)).toBe('2.15 GB');
    });

    it('formats kilobytes correctly with specific format (JEDEC)', () => {
      // 2000 bytes → ~1.95 KB (JEDEC)
      expect(HumanReadableUtils.bytesToString(2000, 'jedec')).toBe('1.95 KB');
    });

    it('formats megabytes correctly with specific format (JEDEC)', () => {
      // 1_048_576 bytes → 1.00 MB (JEDEC)
      expect(HumanReadableUtils.bytesToString(1048576, 'jedec')).toBe('1.00 MB');
    });

    it('formats large values correctly with specific format (JEDEC)', () => {
      // 2,147,483,648 bytes → 2.00 GB (JEDEC)
      expect(HumanReadableUtils.bytesToString(2147483648, 'jedec')).toBe('2.00 GB');
    });
  });
});
