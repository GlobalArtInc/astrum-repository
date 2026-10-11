

import React from 'react';
import { Box, Flex, Text, Badge } from '@radix-ui/themes';
import { ArrowLeft, } from 'lucide-react';

import { SettingsButton } from '../form';

import './StickyActionBar.scss';
import CommonStrings from '../../constants/CommonStrings';
import SharedStrings from '../../constants/SharedStrings';

export interface StickyActionBarProps {
  /** Mode determines button labels and behavior */
  mode: 'create' | 'edit';

  /** Title displayed in the action bar */
  title: string;

  /** Whether the form has unsaved changes */
  isDirty?: boolean;

  /** Whether a save operation is in progress */
  isSaving?: boolean;

  /** Whether delete action is available (edit mode only) */
  canDelete?: boolean;

  /** Handler for back navigation */
  onBack: () => void;

  /** Handler for save/create action */
  onSave: () => void;

  /** Handler for cancel/discard action */
  onCancel: () => void;

  /** Handler for delete action (edit mode only) */
  onDelete?: () => void;

  /** Optional subtitle text (e.g., entity type) */
  subtitle?: string;

  /** Custom back button label (defaults to "Back") */
  backLabel?: string;

  /** Whether save button should be disabled (beyond dirty state) */
  saveDisabled?: boolean;
}

/**
 * StickyActionBar - A sticky header bar for form pages
 *
 * Provides a consistent action bar that remains visible at the top of the
 * content area when scrolling. Follows Salesforce/ServiceNow design patterns.
 *
 * Layout: [Back] Title [Unsaved Badge] [Delete] [Discard/Cancel] [Save/Create]
 *
 * Features:
 * - Sticky positioning at top of content area
 * - Create mode: Create (primary) + Cancel (ghost)
 * - Edit mode: Save (primary, disabled if !isDirty) + Discard (ghost) + Delete (danger ghost)
 * - Unsaved changes badge when isDirty is true
 * - Loading spinner on save button when saving
 * - Light/dark mode support
 *
 * @example
 * ```tsx
 * <StickyActionBar
 *   mode="edit"
 *   title="Edit User: admin"
 *   isDirty={formState.isDirty}
 *   isSaving={isSaving}
 *   canDelete={hasDeletePermission}
 *   onBack={handleBack}
 *   onSave={handleSave}
 *   onCancel={handleDiscard}
 *   onDelete={handleDelete}
 * />
 * ```
 */
export function StickyActionBar({
  mode,
  title,
  isDirty = false,
  isSaving = false,
  canDelete = false,
  onBack,
  onSave,
  onCancel,
  onDelete,
  subtitle,
  backLabel = CommonStrings.ACTIONS.BACK,
  saveDisabled = false,
}: StickyActionBarProps): JSX.Element {
  const isCreate = mode === 'create';
  const primaryLabel = isCreate ? CommonStrings.ACTIONS.CREATE : CommonStrings.ACTIONS.SAVE;
  const secondaryLabel = isCreate ? CommonStrings.ACTIONS.CANCEL : SharedStrings.STICKY_ACTION_BAR.DISCARD;

  // In edit mode, Save is disabled unless there are changes (or explicitly disabled)
  const isPrimaryDisabled = isCreate ? saveDisabled : (!isDirty || saveDisabled);

  return (
    <Box className="sticky-action-bar" data-testid="sticky-action-bar">
      <Flex align="center" justify="between" className="sticky-action-bar__container">
        {/* Left section: Back button + Title */}
        <Flex align="center" gap="3" className="sticky-action-bar__left">
          <SettingsButton
            variant="ghost"
            onClick={onBack}
            icon={ArrowLeft}
            className="sticky-action-bar__back"
            testId="sticky-action-bar-back"
            disabled={isSaving}
          >
            {backLabel}
          </SettingsButton>

          <Box className="sticky-action-bar__title-container">
            <Text size="4" weight="medium" className="sticky-action-bar__title">
              {title}
            </Text>
            {subtitle && (
              <Text size="2" className="sticky-action-bar__subtitle">
                {subtitle}
              </Text>
            )}
          </Box>
        </Flex>

        {/* Right section: Status badge + Action buttons */}
        <Flex align="center" gap="3" className="sticky-action-bar__right">
          {/* Unsaved changes badge */}
          {isDirty && (
            <Badge
              color="amber"
              variant="soft"
              className="sticky-action-bar__dirty-badge"
              data-testid="sticky-action-bar-dirty-badge"
            >
              {SharedStrings.STICKY_ACTION_BAR.UNSAVED_CHANGES}
            </Badge>
          )}

          {/* Delete button (edit mode only, separated from other actions) */}
          {!isCreate && canDelete && onDelete && (
            <SettingsButton
              variant="ghost"
              onClick={onDelete}
              disabled={isSaving}
              className="sticky-action-bar__delete"
              testId="sticky-action-bar-delete"
            >
              {CommonStrings.ACTIONS.DELETE}
            </SettingsButton>
          )}

          {/* Action buttons group */}
          <Flex gap="2" className="sticky-action-bar__actions">
            {/* Cancel/Discard button */}
            <SettingsButton
              variant="ghost"
              onClick={onCancel}
              disabled={isSaving}
              testId="sticky-action-bar-cancel"
            >
              {secondaryLabel}
            </SettingsButton>

            {/* Save/Create button */}
            <SettingsButton
              variant="primary"
              onClick={onSave}
              disabled={isPrimaryDisabled || isSaving}
              loading={isSaving}
              testId="sticky-action-bar-save"
            >
              {primaryLabel}
            </SettingsButton>
          </Flex>
        </Flex>
      </Flex>
    </Box>
  );
}

export default StickyActionBar;
