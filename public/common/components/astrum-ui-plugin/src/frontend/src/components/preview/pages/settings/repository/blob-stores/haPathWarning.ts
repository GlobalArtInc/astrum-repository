

export const HA_PATH_WARNING = {
  TITLE: 'High Availability Path Warning',
  MESSAGE: 'Using a relative path or a path under the working directory in HA mode can cause ' +
    'severe performance issues and data inconsistency. The path may not be shared between HA pods, leading to ' +
    'random request failures. Additionally, initContainer recursive operations can cause severe I/O bottlenecks ' +
    'and startup delays (10+ minutes). It is strongly recommended to use an absolute shared path outside the ' +
    'working directory for file blob stores in HA deployments.'
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
