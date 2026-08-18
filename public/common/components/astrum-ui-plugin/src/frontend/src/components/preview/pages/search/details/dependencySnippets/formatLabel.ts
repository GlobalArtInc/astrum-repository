

import { FORMAT_LABELS } from '../../../../shared/Badges/formatConstants';
import { FORMAT_ALIASES } from './registry';

/**
 * Human-readable label for a repository format, reusing the shared FORMAT_LABELS map.
 *
 * Falls back to the raw format id when the format has no known label, so a newly added
 * format still renders something rather than nothing.
 */
export function getFormatLabel(format: string): string {
  const raw = format ?? '';
  // Lower-case before the alias/label lookup, matching normalizeFormatKey, so a format id
  // arriving in mixed case (e.g. "MAVEN") still resolves to its canonical label.
  const normalized = raw.toLowerCase();
  const key = FORMAT_ALIASES[normalized] ?? normalized;
  return FORMAT_LABELS[key] ?? raw;
}
