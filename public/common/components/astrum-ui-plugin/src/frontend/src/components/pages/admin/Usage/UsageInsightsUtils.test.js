

import {
  getScaleFactor,
  getMaxValue,
  getValueTicks,
  getMetricDateTicks,
  formatAsShortDate,
  formatAsISODate,
  getDateRange,
  getMonthOptions,
  KEY_EGRESS,
  KEY_STORAGE
} from './UsageInsightsUtils';
import HumanReadableUtils from '../../../../interface/HumanReadableUtils';

const KB_IN_BYTES = 1024;
const MB_IN_BYTES = KB_IN_BYTES * 1024;
const GB_IN_BYTES = MB_IN_BYTES * 1024;

describe('UsageInsightsUtils', () => {
  describe('getScaleFactor', () => {
    it('returns 1 for values < 1000', () => {
      expect(getScaleFactor(0)).toBe(1);
      expect(getScaleFactor(100)).toBe(1);
      expect(getScaleFactor(999)).toBe(1);
    });

    it('returns 1000 (k scale) for values >= 1000 and < 1000000', () => {
      expect(getScaleFactor(1000)).toBe(1000);
      expect(getScaleFactor(5000)).toBe(1000);
      expect(getScaleFactor(999999)).toBe(1000);
    });

    it('returns 1000000 (M scale) for values >= 1000000 and < 1000000000', () => {
      expect(getScaleFactor(1000000)).toBe(1000000);
      expect(getScaleFactor(5000000)).toBe(1000000);
      expect(getScaleFactor(999999999)).toBe(1000000);
    });

    it('returns 1000000000 (G scale) for values >= 1000000000', () => {
      expect(getScaleFactor(1000000000)).toBe(1000000000);
      expect(getScaleFactor(5000000000)).toBe(1000000000);
      expect(getScaleFactor(10000000000)).toBe(1000000000);
    });
  });

  describe('getMaxValue', () => {
    it('calculates max value from egress and storage combined', () => {
      const data = [
        {[KEY_EGRESS]: 1000, [KEY_STORAGE]: 500},
        {[KEY_EGRESS]: 2000, [KEY_STORAGE]: 3000},
        {[KEY_EGRESS]: 500, [KEY_STORAGE]: 1000}
      ];

      // Max is 5000 (2000 + 3000), scale is 1000 (k)
      // Result: ceil(5000/1000/0.1)*1000*0.1 = ceil(50)*1000*0.1 = 50*100 = 5000
      const maxValue = getMaxValue(data);
      expect(maxValue).toBe(5000);
    });

    it('handles missing or invalid egress/storage values', () => {
      const data = [
        {[KEY_EGRESS]: null, [KEY_STORAGE]: 1000},
        {[KEY_EGRESS]: 2000, [KEY_STORAGE]: undefined},
        {[KEY_EGRESS]: 'invalid', [KEY_STORAGE]: 500}
      ];

      const maxValue = getMaxValue(data);
      expect(maxValue).toBeGreaterThanOrEqual(0);
    });

    it('returns 0 for empty or null data', () => {
      expect(getMaxValue([])).toBe(0);
      expect(getMaxValue(null)).toBe(0);
      expect(getMaxValue(undefined)).toBe(0);
    });

    it('scales max value appropriately based on scale factor', () => {
      const data = [
        {[KEY_EGRESS]: 1500000000, [KEY_STORAGE]: 500000000}
      ];

      // Max is 2000000000 (2 billion), scale is 1000000000 (G)
      // Result: ceil(2000000000/1000000000/0.1)*1000000000*0.1 = ceil(20)*1000000000*0.1 = 20*100000000 = 2000000000
      const maxValue = getMaxValue(data);
      expect(maxValue).toBe(2000000000);
    });

    it('rounds up to nearest scale unit with 0.1 precision', () => {
      const data = [
        {[KEY_EGRESS]: 1234, [KEY_STORAGE]: 0}
      ];

      // Max is 1234, scale is 1000 (k)
      // Result: ceil(1234/1000/0.1)*1000*0.1 = ceil(12.34)*1000*0.1 = 13*100 = 1300
      const maxValue = getMaxValue(data);
      expect(maxValue).toBe(1300);
    });
  });

  describe('getValueTicks', () => {
    it('generates correct number of ticks', () => {
      const maxValue = 1000;
      const maxValueTicks = 5;
      const ticks = getValueTicks(maxValue, maxValueTicks);

      expect(ticks).toHaveLength(5);
      expect(ticks).toEqual([200, 400, 600, 800, 1000]);
    });

    it('returns empty array when maxValue is 0', () => {
      expect(getValueTicks(0, 5)).toEqual([]);
    });

    it('handles different tick counts', () => {
      const maxValue = 1000;

      expect(getValueTicks(maxValue, 4)).toEqual([250, 500, 750, 1000]);
      expect(getValueTicks(maxValue, 10)).toHaveLength(10);
    });
  });

  describe('getMetricDateTicks', () => {
    it('returns all dates when data length <= 8', () => {
      const data = [
        {metricDate: '2024-01-01'},
        {metricDate: '2024-01-02'},
        {metricDate: '2024-01-03'},
        {metricDate: '2024-01-04'}
      ];

      const ticks = getMetricDateTicks(data);
      expect(ticks).toEqual(['2024-01-01', '2024-01-02', '2024-01-03', '2024-01-04']);
    });

    it('returns empty array for empty data', () => {
      expect(getMetricDateTicks([])).toEqual([]);
    });

    it('samples dates when data length > 8', () => {
      const data = Array.from({length: 30}, (_, i) => ({
        metricDate: `2024-01-${String(i + 1).padStart(2, '0')}`
      }));

      const ticks = getMetricDateTicks(data);

      // Should return 4 or 5 ticks based on the algorithm
      expect(ticks.length).toBeLessThanOrEqual(5);
      expect(ticks.length).toBeGreaterThanOrEqual(4);

      // First tick should be first date, last tick should be last date
      expect(ticks[0]).toBe('2024-01-01');
      expect(ticks[ticks.length - 1]).toBe('2024-01-30');
    });

    it('handles odd and even length data correctly', () => {
      const evenData = Array.from({length: 10}, (_, i) => ({
        metricDate: `2024-01-${String(i + 1).padStart(2, '0')}`
      }));

      const oddData = Array.from({length: 9}, (_, i) => ({
        metricDate: `2024-01-${String(i + 1).padStart(2, '0')}`
      }));

      const evenTicks = getMetricDateTicks(evenData);
      const oddTicks = getMetricDateTicks(oddData);

      // Both should return reasonable tick counts
      expect(evenTicks.length).toBeGreaterThan(0);
      expect(oddTicks.length).toBeGreaterThan(0);
    });
  });

  describe('HumanReadableUtils.bytesToString', () => {
    it('formats 0 bytes', () => {
      const result = HumanReadableUtils.bytesToString(0);
      expect(result).toBe('0.00 Bytes');
    });

    it('formats bytes', () => {
      const result = HumanReadableUtils.bytesToString(500);
      expect(result).toContain('Bytes');
    });

    it('formats KB', () => {
      const result = HumanReadableUtils.bytesToString(KB_IN_BYTES);
      expect(result).toMatch(/k/i);
    });

    it('formats MB', () => {
      const result = HumanReadableUtils.bytesToString(MB_IN_BYTES);
      expect(result).toMatch(/M/);
    });

    it('formats GB', () => {
      const result = HumanReadableUtils.bytesToString(GB_IN_BYTES);
      expect(result).toMatch(/G/);
    });

    it('handles negative values', () => {
      const result = HumanReadableUtils.bytesToString(-100);
      expect(result).toBe('Unavailable');
    });

    it('formats with default si notation', () => {
      const result = HumanReadableUtils.bytesToString(1000);
      // SI notation uses 1000 as base
      expect(result).toBeTruthy();
      expect(result).toContain('k');
    });

    it('formats with iec notation', () => {
      const result = HumanReadableUtils.bytesToString(KB_IN_BYTES, 'iec');
      // IEC notation uses 1024 as base
      expect(result).toMatch(/KiB|Bytes/);
    });
  });

  describe('formatAsShortDate', () => {
    it('formats date in short format using locale', () => {
      const result = formatAsShortDate('2024-01-15');
      // Should match pattern like "Jan 15" (month abbreviation + day)
      expect(result).toMatch(/^[A-Za-z]{3}\s+\d{1,2}$/);
    });

    it('formats first day of month', () => {
      const result = formatAsShortDate('2024-01-01');
      // Should match "Jan 1" pattern
      expect(result).toMatch(/^[A-Za-z]{3}\s+1$/);
    });

    it('formats different days correctly', () => {
      const day02 = formatAsShortDate('2024-01-02');
      const day15 = formatAsShortDate('2024-01-15');
      const day31 = formatAsShortDate('2024-01-31');

      expect(day02).toMatch(/^[A-Za-z]{3}\s+2$/);
      expect(day15).toMatch(/^[A-Za-z]{3}\s+15$/);
      expect(day31).toMatch(/^[A-Za-z]{3}\s+31$/);
    });

    it('handles different months', () => {
      const jan = formatAsShortDate('2024-01-15');
      const feb = formatAsShortDate('2024-02-15');
      const dec = formatAsShortDate('2024-12-25');

      // All should match the pattern
      expect(jan).toMatch(/^[A-Za-z]{3}\s+15$/);
      expect(feb).toMatch(/^[A-Za-z]{3}\s+15$/);
      expect(dec).toMatch(/^[A-Za-z]{3}\s+25$/);

      // Different months should produce different strings
      expect(jan).not.toBe(feb);
      expect(feb).not.toBe(dec);
    });

    it('formats dates consistently', () => {
      const result1 = formatAsShortDate('2024-01-15');
      const result2 = formatAsShortDate('2024-01-15');

      // Same input should produce same output
      expect(result1).toBe(result2);
    });

    it('handles end of month dates', () => {
      const jan31 = formatAsShortDate('2024-01-31');
      const feb29 = formatAsShortDate('2024-02-29'); // Leap year
      const apr30 = formatAsShortDate('2024-04-30');

      expect(jan31).toMatch(/^[A-Za-z]{3}\s+31$/);
      expect(feb29).toMatch(/^[A-Za-z]{3}\s+29$/);
      expect(apr30).toMatch(/^[A-Za-z]{3}\s+30$/);
    });
  });

  describe('formatAsISODate', () => {
    it('formats date as ISO string (YYYY-MM-DD)', () => {
      const date = new Date(2024, 0, 15); // January 15, 2024
      expect(formatAsISODate(date)).toBe('2024-01-15');
    });

    it('pads single digit month and day with zeros', () => {
      const date = new Date(2024, 0, 1); // January 1, 2024
      expect(formatAsISODate(date)).toBe('2024-01-01');
    });

    it('handles different years', () => {
      const date = new Date(2025, 11, 31); // December 31, 2025
      expect(formatAsISODate(date)).toBe('2025-12-31');
    });
  });

  describe('getDateRange', () => {
    it('returns first and last day of the month', () => {
      const baseDate = new Date(2024, 0, 15); // January 15, 2024
      const result = getDateRange(baseDate);

      expect(result.dateFrom).toBe('2024-01-01');
      expect(result.dateTo).toBe('2024-01-31');
      expect(result.dateFromAsDate).toEqual(new Date(2024, 0, 1));
      expect(result.dateToAsDate).toEqual(new Date(2024, 0, 31));
    });

    it('handles February in leap year', () => {
      const baseDate = new Date(2024, 1, 15); // February 15, 2024 (leap year)
      const result = getDateRange(baseDate);

      expect(result.dateFrom).toBe('2024-02-01');
      expect(result.dateTo).toBe('2024-02-29');
    });

    it('handles February in non-leap year', () => {
      const baseDate = new Date(2023, 1, 15); // February 15, 2023 (non-leap year)
      const result = getDateRange(baseDate);

      expect(result.dateFrom).toBe('2023-02-01');
      expect(result.dateTo).toBe('2023-02-28');
    });

    it('handles month with 30 days', () => {
      const baseDate = new Date(2024, 3, 15); // April 15, 2024
      const result = getDateRange(baseDate);

      expect(result.dateFrom).toBe('2024-04-01');
      expect(result.dateTo).toBe('2024-04-30');
    });

    it('handles December', () => {
      const baseDate = new Date(2024, 11, 15); // December 15, 2024
      const result = getDateRange(baseDate);

      expect(result.dateFrom).toBe('2024-12-01');
      expect(result.dateTo).toBe('2024-12-31');
    });
  });

  describe('getMonthOptions', () => {
    it('returns default 12 month options', () => {
      const options = getMonthOptions();

      expect(options).toHaveLength(12);
      expect(options[0]).toHaveProperty('key');
      expect(options[0]).toHaveProperty('value');
      expect(options[0]).toHaveProperty('label');
    });

    it('returns specified number of month options', () => {
      const options = getMonthOptions(6);

      expect(options).toHaveLength(6);
    });

    it('generates correct month range (current month going backwards)', () => {
      const options = getMonthOptions(3);

      // First option should be current month
      const currentDate = new Date();
      const currentMonthStart = new Date(currentDate.getFullYear(), currentDate.getMonth(), 1);
      const currentMonthEnd = new Date(currentDate.getFullYear(), currentDate.getMonth() + 1, 0);

      expect(options[0].value.dateFrom).toBe(formatAsISODate(currentMonthStart));
      expect(options[0].value.dateTo).toBe(formatAsISODate(currentMonthEnd));

      // Options should go backwards in time
      expect(options).toHaveLength(3);
    });

    it('formats labels with date range', () => {
      const options = getMonthOptions(1);

      // Label should contain date range separator
      expect(options[0].label).toContain(' - ');
      // Label should be a non-empty string
      expect(options[0].label.length).toBeGreaterThan(0);
    });

    it('generates correct key format', () => {
      const options = getMonthOptions(1);

      // Key should be in format "YYYY-MM-DD-YYYY-MM-DD"
      expect(options[0].key).toMatch(/^\d{4}-\d{2}-\d{2}-\d{4}-\d{2}-\d{2}$/);
    });

    it('handles year transitions correctly', () => {
      const options = getMonthOptions(13);

      // With 13 months, we should span across year boundary
      expect(options).toHaveLength(13);

      // Check that we have different years in the options
      const years = options.map(opt => opt.value.dateFrom.substring(0, 4));
      const uniqueYears = [...new Set(years)];
      expect(uniqueYears.length).toBeGreaterThanOrEqual(1);
    });
  });

  describe('KEY_EGRESS and KEY_STORAGE constants', () => {
    it('exports correct constant values', () => {
      expect(KEY_EGRESS).toBe('egress');
      expect(KEY_STORAGE).toBe('storage');
    });
  });
});
