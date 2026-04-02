import { useMemo } from 'react';
import { useApi } from '../../api';
import { useSession } from '../../auth/state';
import { formatName } from '../../ui';
import { REPOSITORIES, type RepositorySummary } from './model';

export function useFormatOptions(): Array<{ value: string; label: string }> {
  const { state } = useSession();
  return useMemo(
    () =>
      state.formats
        .map((format) => ({ value: format.id, label: formatName(format.id) }))
        .sort((left, right) => left.label.localeCompare(right.label)),
    [state.formats],
  );
}

export function useRepositoryNames(format: string, keep = ''): string[] {
  const { data } = useApi<RepositorySummary[]>(REPOSITORIES);
  return useMemo(() => {
    const names = (data ?? [])
      .filter((repository) => format === '' || repository.format === format)
      .map((repository) => repository.name);
    if (keep !== '' && !names.includes(keep)) {
      names.push(keep);
    }
    return names.sort((left, right) => left.localeCompare(right));
  }, [data, format, keep]);
}
