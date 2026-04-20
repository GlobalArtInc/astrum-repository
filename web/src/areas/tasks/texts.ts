import { useT } from '../../i18n';
import type { ProblemTexts } from './model';

export function useProblemTexts(): ProblemTexts {
  const t = useT();
  return {
    required: (label) => t('tasks.problems.required', { label }),
    number: (label) => t('tasks.problems.number', { label }),
    min: (label, min) => t('tasks.problems.min', { label, min }),
    max: (label, max) => t('tasks.problems.max', { label, max }),
    email: t('tasks.problems.email'),
    name: t('tasks.problems.name'),
    days: t('tasks.problems.days'),
    date: t('tasks.problems.date'),
    time: t('tasks.problems.time'),
    cron: t('tasks.problems.cron'),
    cronParts: t('tasks.problems.cronParts'),
    cronDay: t('tasks.problems.cronDay'),
  };
}
