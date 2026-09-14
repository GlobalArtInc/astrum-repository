

import { isRelativePath, isUnderWorkDirectory, shouldShowHaPathWarning } from '../haPathWarning';

describe('haPathWarning', () => {
  describe('isRelativePath', () => {
    it('returns false for null path', () => {
      expect(isRelativePath(null)).toBe(false);
    });

    it('returns false for undefined path', () => {
      expect(isRelativePath(undefined)).toBe(false);
    });

    it('returns false for empty string', () => {
      expect(isRelativePath('')).toBe(false);
    });

    it('returns false for unix absolute path', () => {
      expect(isRelativePath('/data/blobs')).toBe(false);
    });

    it('returns false for windows absolute path', () => {
      expect(isRelativePath('C:\\data\\blobs')).toBe(false);
    });

    it('returns false for windows absolute path with lowercase drive', () => {
      expect(isRelativePath('d:\\storage')).toBe(false);
    });

    it('returns true for relative path without leading slash', () => {
      expect(isRelativePath('blobs/default')).toBe(true);
    });

    it('returns true for single directory name', () => {
      expect(isRelativePath('default')).toBe(true);
    });

    it('returns true for dot-relative path', () => {
      expect(isRelativePath('./blobs')).toBe(true);
    });

    it('returns true for parent-relative path', () => {
      expect(isRelativePath('../blobs')).toBe(true);
    });
  });

  describe('isUnderWorkDirectory', () => {
    const WORK_DIR = '/astrum-data';

    it('returns false for null path', () => {
      expect(isUnderWorkDirectory(null, WORK_DIR)).toBe(false);
    });

    it('returns false for undefined path', () => {
      expect(isUnderWorkDirectory(undefined, WORK_DIR)).toBe(false);
    });

    it('returns false for null work directory', () => {
      expect(isUnderWorkDirectory('/astrum-data/blobs', null)).toBe(false);
    });

    it('returns false for undefined work directory', () => {
      expect(isUnderWorkDirectory('/astrum-data/blobs', undefined)).toBe(false);
    });

    it('returns false for empty work directory', () => {
      expect(isUnderWorkDirectory('/astrum-data/blobs', '')).toBe(false);
    });

    it('returns true when path is directly under work directory', () => {
      expect(isUnderWorkDirectory('/astrum-data/blobs', WORK_DIR)).toBe(true);
    });

    it('returns true when path is deeply nested under work directory', () => {
      expect(isUnderWorkDirectory('/astrum-data/blobs/default/content', WORK_DIR)).toBe(true);
    });

    it('returns false when path is outside work directory', () => {
      expect(isUnderWorkDirectory('/mnt/shared/blobs', WORK_DIR)).toBe(false);
    });

    it('returns false for path that starts with same prefix but is different directory', () => {
      expect(isUnderWorkDirectory('/astrum-data-backup/blobs', WORK_DIR)).toBe(false);
    });

    it('handles work directory with trailing slash', () => {
      expect(isUnderWorkDirectory('/astrum-data/blobs', '/astrum-data/')).toBe(true);
    });

    it('normalizes windows backslashes in path', () => {
      expect(isUnderWorkDirectory('C:\\astrum-data\\blobs', 'C:\\astrum-data')).toBe(true);
    });

    it('normalizes windows backslashes in work directory', () => {
      expect(isUnderWorkDirectory('C:/astrum-data/blobs', 'C:\\astrum-data')).toBe(true);
    });

    it('returns false when path equals work directory exactly', () => {
      expect(isUnderWorkDirectory('/astrum-data', WORK_DIR)).toBe(false);
    });
  });

  describe('shouldShowHaPathWarning', () => {
    const WORK_DIR = '/astrum-data';

    it('returns true for relative path', () => {
      expect(shouldShowHaPathWarning('blobs/default', WORK_DIR)).toBe(true);
    });

    it('returns true for path under work directory', () => {
      expect(shouldShowHaPathWarning('/astrum-data/blobs', WORK_DIR)).toBe(true);
    });

    it('returns false for absolute path outside work directory', () => {
      expect(shouldShowHaPathWarning('/mnt/shared/blobs', WORK_DIR)).toBe(false);
    });

    it('returns false for null path', () => {
      expect(shouldShowHaPathWarning(null, WORK_DIR)).toBe(false);
    });

    it('returns true when path is relative even with null work directory', () => {
      expect(shouldShowHaPathWarning('blobs/default', null)).toBe(true);
    });

    it('returns false for absolute path outside work directory with null work directory', () => {
      expect(shouldShowHaPathWarning('/mnt/shared/blobs', null)).toBe(false);
    });
  });
});
