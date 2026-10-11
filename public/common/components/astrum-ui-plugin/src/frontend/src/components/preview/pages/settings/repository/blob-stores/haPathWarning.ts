import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';


export const HA_PATH_WARNING = {
  TITLE: BlobStoresStrings.HA_PATH_WARNING.HIGH_AVAILABILITY_PATH_WARNING,
  MESSAGE: BlobStoresStrings.HA_PATH_WARNING.USING_RELATIVE_PATH_PATH_UNDER
};

export function isRelativePath(path: string | undefined | null): boolean {
  if (!path) {
    return false;
  }
  return !path.startsWith('/') && !/^[A-Za-z]:\\/.test(path);
}

export function isUnderWorkDirectory(path: string | undefined | null, workDirectory: string | undefined | null): boolean {
  if (!path || !workDirectory) {
    return false;
  }
  const normalizedPath = path.replace(/\\/g, '/');
  let normalizedWorkDir = workDirectory.replace(/\\/g, '/');

  if (!normalizedWorkDir.endsWith('/')) {
    normalizedWorkDir += '/';
  }

  return normalizedPath.startsWith(normalizedWorkDir);
}

export function shouldShowHaPathWarning(path: string | undefined | null, workDirectory: string | undefined | null): boolean {
  return isRelativePath(path) || isUnderWorkDirectory(path, workDirectory);
}
