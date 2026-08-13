

/**
 * Browse Actions Module
 *
 * Provides action components for the Browse functionality:
 * - DeleteDialog: Confirmation dialog for delete operations
 * - CopyUrlButton: Copy URL to clipboard with toast feedback
 * - useActions: Hook for delete API operations
 */

// Components
export { DeleteDialog } from './DeleteDialog';
export { CopyUrlButton } from './CopyUrlButton';

// Hooks
export { useActions } from './useActions';

// Types
export type {
  DeleteItemType,
  DeleteItemInfo,
  DeleteDialogProps,
  CopyUrlButtonProps,
  DeleteResult,
  UseActionsOptions,
  UseActionsReturn,
  ActionStrings,
} from './actions.types';

// Constants
export { DELETE_ENDPOINTS, ACTION_STRINGS } from './actions.types';

