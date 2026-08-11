

import {
  Loader2,
  AlertCircle,
  AlertTriangle,
  CheckCircle2,
  Info,
} from 'lucide-react';

/**
 * Semantic status icon mappings for feedback states.
 *
 * @example
 * import { StatusIcons } from '@nosc/icons/status-icons';
 * <StatusIcons.Loading size={16} className="animate-spin" />
 */
export const StatusIcons = {
  Loading: Loader2,
  Error: AlertCircle,
  Warning: AlertTriangle,
  Success: CheckCircle2,
  Info,
} as const;

export type StatusIconName = keyof typeof StatusIcons;
