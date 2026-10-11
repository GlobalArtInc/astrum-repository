

import React, { useCallback, useEffect, useState } from 'react';
import { Badge, Box, Button, Flex, Heading, Link, Text } from '@radix-ui/themes';
import { ChevronRight, Loader2, ExternalLink } from 'lucide-react';
import { useRouter } from '@uirouter/react';

import { useToast, ConfirmDialog, Tooltip } from '../../../../shared';
import { RecoveryModeTasksTable } from './RecoveryModeTasksTable';
import { useRecoveryModeApi } from './useRecoveryModeApi';
import { RecoveryModeData } from './types';

import './RecoveryModePage.scss';
import RecoveryModeStrings from '../../../../constants/pages/settings/support/RecoveryModeStrings';
import CommonStrings from '../../../../constants/CommonStrings';

const LEARN_MORE_URL = 'https://links.globalart.dev/products/astrum/docs/recovery-mode';
/** UI-Router state name for the Settings hub (breadcrumb target). */
const SETTINGS_STATE = 'preview.admin.settings';

const STRINGS = {
  TITLE: RecoveryModeStrings.PAGE.RECOVERY_MODE,
  DESCRIPTION:
    RecoveryModeStrings.PAGE.WHILE_RECOVERY_MODE_ENABLED_SYSTEM,
  ENABLE: RecoveryModeStrings.PAGE.ENABLE,
  DISABLE: RecoveryModeStrings.PAGE.DISABLE,
  ENABLED: RecoveryModeStrings.PAGE.ENABLED,
  DISABLED: RecoveryModeStrings.PAGE.DISABLED,
  DISABLE_RUNNING_TOOLTIP:
    RecoveryModeStrings.PAGE.RECONCILIATION_TASKS_STILL_RUNNING_WAIT,
  CONFIRM: {
    TITLE: RecoveryModeStrings.PAGE.DISABLE_RECOVERY_MODE_UNEXECUTED_PLANS,
    MESSAGE:
      RecoveryModeStrings.PAGE.ONE_MORE_DATA_REPAIR_PLANS,
    CONFIRM_LABEL: RecoveryModeStrings.PAGE.DISABLE_MODE,
    CANCEL_LABEL: CommonStrings.ACTIONS.CANCEL,
  },
};

function hasRunningTask(data: RecoveryModeData | null): boolean {
  return !!data?.reconcileTasks?.some((t) => t.currentState?.toUpperCase().startsWith('RUNNING'));
}

export interface RecoveryModePageProps {
  className?: string;
}

/**
 * RecoveryModePage - Recovery Mode administration page (Preview UI).
 *
 * Shows current state, an enable/disable toggle, and the data repair tasks
 * table. Disable is blocked while a repair task is running, and prompts for
 * confirmation when there are unexecuted plans.
 */
export function RecoveryModePage({ className }: RecoveryModePageProps) {
  const { error, setError, fetchRecoveryMode, enableRecoveryMode, disableRecoveryMode } =
    useRecoveryModeApi();
  const toast = useToast();
  const router = useRouter();

  const [data, setData] = useState<RecoveryModeData | null>(null);
  const [loadingInitial, setLoadingInitial] = useState(true);
  const [busy, setBusy] = useState(false);
  const [showDisableConfirm, setShowDisableConfirm] = useState(false);

  const loadData = useCallback(async () => {
    try {
      const result = await fetchRecoveryMode();
      setData(result);
    } catch {
      // error is set by the hook
    }
  }, [fetchRecoveryMode]);

  useEffect(() => {
    (async () => {
      setLoadingInitial(true);
      await loadData();
      setLoadingInitial(false);
    })();
  }, [loadData]);

  const handleEnable = useCallback(async () => {
    setBusy(true);
    try {
      await enableRecoveryMode();
      await loadData();
      toast.success(RecoveryModeStrings.PAGE.RECOVERY_MODE_ENABLED);
    } catch {
      toast.error(RecoveryModeStrings.PAGE.FAILED_ENABLE_RECOVERY_MODE);
    } finally {
      setBusy(false);
    }
  }, [enableRecoveryMode, loadData, toast]);

  const performDisable = useCallback(async () => {
    setBusy(true);
    try {
      await disableRecoveryMode();
      await loadData();
      toast.success(RecoveryModeStrings.PAGE.RECOVERY_MODE_DISABLED);
    } catch {
      toast.error(RecoveryModeStrings.PAGE.FAILED_DISABLE_RECOVERY_MODE);
    } finally {
      setBusy(false);
    }
  }, [disableRecoveryMode, loadData, toast]);

  const handleDisableClick = useCallback(() => {
    // Confirm first when there are unexecuted plans; otherwise disable directly.
    if (data?.unexecutedPlans) {
      setShowDisableConfirm(true);
    } else {
      performDisable();
    }
  }, [data, performDisable]);

  // Loading state
  if (loadingInitial) {
    return (
      <Box className={`recovery-mode-page ${className || ''}`.trim()} data-testid="recovery-mode-page">
        <Flex align="center" justify="center" gap="2" className="recovery-mode-page__loading">
          <Loader2 size={20} className="recovery-mode-page__spinner" />
          <Text size="2">{RecoveryModeStrings.PAGE.LOADING_RECOVERY_MODE}</Text>
        </Flex>
      </Box>
    );
  }

  const enabled = !!data?.enabled;
  const running = hasRunningTask(data);
  // The initial fetch failed (no state loaded). Don't let the user toggle
  // recovery mode against an unknown prior state — disable the actions.
  const loadFailed = data === null;

  const disableButton = (
    <Button
      variant="solid"
      onClick={handleDisableClick}
      disabled={running || busy || loadFailed}
      data-testid="recovery-mode-disable"
    >
      {STRINGS.DISABLE}
    </Button>
  );

  const actions = enabled ? (
    // A disabled button doesn't emit hover events, so when it's disabled because
    // a task is running, wrap it in a span (which does) and show the tooltip.
    running ? (
      <Tooltip content={STRINGS.DISABLE_RUNNING_TOOLTIP}>
        <span
          className="recovery-mode-page__disable-wrap"
          // biome-ignore lint/a11y/noNoninteractiveTabindex: the disabled Button emits no focus/hover events, so this wrapper must be focusable for keyboard users to reveal the "why is this disabled" tooltip.
          tabIndex={0}
          data-testid="recovery-mode-disable-tooltip"
        >
          {disableButton}
        </span>
      </Tooltip>
    ) : (
      disableButton
    )
  ) : (
    <Button variant="solid" onClick={handleEnable} disabled={busy || loadFailed} data-testid="recovery-mode-enable">
      {STRINGS.ENABLE}
    </Button>
  );

  return (
    <Box className={`recovery-mode-page ${className || ''}`.trim()} data-testid="recovery-mode-page">
      {/* Breadcrumbs */}
      <Flex align="center" gap="1" mb="2" className="recovery-mode-page__breadcrumbs">
        <Button
          variant="ghost"
          size="2"
          color="blue"
          onClick={() => router.stateService.go(SETTINGS_STATE)}
        >
          {CommonStrings.LABELS.SETTINGS}
        </Button>
        <ChevronRight size={14} />
        <Text size="2" color="gray">{STRINGS.TITLE}</Text>
      </Flex>

      {/* Title row: heading + status badge, actions on the right */}
      <Flex align="start" justify="between" gap="4" className="recovery-mode-page__header">
        <Box>
          <Flex align="center" gap="3" mb="1">
            <Heading size="6">{STRINGS.TITLE}</Heading>
            <Badge
              color={enabled ? 'green' : 'gray'}
              radius="small"
              className="recovery-mode-page__status-badge"
              data-testid="recovery-mode-status"
            >
              {enabled ? STRINGS.ENABLED : STRINGS.DISABLED}
            </Badge>
          </Flex>
          <Box className="recovery-mode-page__description">
            <Text size="2" color="gray">{STRINGS.DESCRIPTION}</Text>{' '}
            <Link
              size="2"
              href={LEARN_MORE_URL}
              target="_blank"
              rel="noopener noreferrer"
              color="blue"
              className="recovery-mode-page__learn-more"
            >
              {CommonStrings.ACTIONS.LEARN_MORE}<ExternalLink size={12} />
            </Link>
          </Box>
        </Box>
        <Box className="recovery-mode-page__actions">{actions}</Box>
      </Flex>

      {error && (
        <Box mb="3" className="recovery-mode-page__error" role="alert">
          <Text size="2" color="red">{error}</Text>
        </Box>
      )}

      <RecoveryModeTasksTable tasks={data?.reconcileTasks ?? []} />

      <ConfirmDialog
        open={showDisableConfirm}
        onOpenChange={setShowDisableConfirm}
        title={STRINGS.CONFIRM.TITLE}
        message={STRINGS.CONFIRM.MESSAGE}
        confirmLabel={STRINGS.CONFIRM.CONFIRM_LABEL}
        cancelLabel={STRINGS.CONFIRM.CANCEL_LABEL}
        variant="warning"
        onConfirm={performDisable}
      />
    </Box>
  );
}

export default RecoveryModePage;
