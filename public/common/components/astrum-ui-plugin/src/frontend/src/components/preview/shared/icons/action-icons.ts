

import {
  Trash2,
  Plus,
  ArrowLeft,
  Pencil,
  Check,
  X,
  Search,
  RefreshCw,
  Download,
  ExternalLink,
  Copy,
  Settings2,
} from 'lucide-react';

/**
 * Semantic action icon mappings.
 *
 * Use these instead of importing Lucide icons directly to ensure
 * consistent icon usage across the application.
 *
 * @example
 * import { ActionIcons } from '@nosc/icons/action-icons';
 * <ActionIcons.Delete size={16} />
 */
export const ActionIcons = {
  Delete: Trash2,
  Add: Plus,
  Back: ArrowLeft,
  Edit: Pencil,
  Save: Check,
  Cancel: X,
  Search,
  Refresh: RefreshCw,
  Download,
  ExternalLink,
  Copy,
  Settings: Settings2,
} as const;

export type ActionIconName = keyof typeof ActionIcons;
