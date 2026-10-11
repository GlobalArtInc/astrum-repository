

import React, { useState, useCallback, useMemo, useRef, useEffect } from 'react';
import { Box, Flex, Text, Heading, Badge, Tabs, Skeleton } from '@radix-ui/themes';
import {
  Trash2, Play, Square, FileText, Settings, Calendar, History, Edit3,
  CheckCircle, XCircle, AlertCircle, Clock, Mail, Info, Puzzle, RefreshCw,
} from 'lucide-react';

import {
  SettingsForm,
  SettingsFormSection,
  SettingsTextInput,
  SettingsToggle,
  SettingsSelect,
  SettingsButton,
  ConfirmDialog,
} from '../../../../shared/form';
import { DeleteConfirmationModal } from '../../../../shared/modals/DeleteConfirmationModal';
import { TaskScheduler } from './TaskScheduler';
import { DynamicFormFields } from './TaskTypeSelector';
import { TaskHistory } from './TaskHistory';
import { isManualOnlyTaskType } from './taskFieldMetadata';
import { useTasksForm } from './useTasksForm';
import { useTasksApi } from './useTasksApi';
import {
  Task,
  TaskFormData,
  TaskDetailProps,
  ScheduleData,
  ScheduleType,
  formatDate,
  NOTIFICATION_CONDITIONS,
} from './types';

import './TaskDetail.scss';
import CommonStrings from '../../../../constants/CommonStrings';
import TasksStrings from '../../../../constants/pages/settings/system/TasksStrings';

function getStatusBadgeColor(status: string): 'blue' | 'green' | 'gray' | 'yellow' | 'red' {
  switch (status) {
    case 'RUNNING': return 'blue';
    case 'OK': return 'green';
    case 'WAITING': return 'gray';
    case 'BLOCKED': return 'yellow';
    default: return 'red';
  }
}

function getStatusIcon(status: string) {
  switch (status) {
    case 'RUNNING': return <RefreshCw size={16} className="task-detail__status-icon task-detail__status-icon--running" />;
    case 'OK': return <CheckCircle size={16} className="task-detail__status-icon task-detail__status-icon--ok" />;
    case 'WAITING': return <Clock size={16} className="task-detail__status-icon task-detail__status-icon--waiting" />;
    case 'BLOCKED': return <AlertCircle size={16} className="task-detail__status-icon task-detail__status-icon--blocked" />;
    default: return <XCircle size={16} className="task-detail__status-icon task-detail__status-icon--error" />;
  }
}

// Summary item icon mapping
const _SUMMARY_ICONS: Record<string, React.ComponentType<{ size?: number; className?: string }>> = {
  status: CheckCircle,
  lastResult: FileText,
  schedule: Calendar,
  nextRun: Clock,
  lastRun: History,
  alertEmail: Mail,
  description: Info,
};

export function TaskDetail({
  task: initialTask,
  liveTask,
  taskId: taskIdFromRoute,
  loading: initialLoading = false,
  canEdit = false,
  canDelete = false,
  canRun = false,
  canStop = false,
  onSave: onSaveProp,
  onDelete: onDeleteProp,
  onRun,
  onStop,
  onCancel,
  error: externalError,
}: TaskDetailProps) {
  const { createTask, updateTask, deleteTask } = useTasksApi();
  const [activeTab, setActiveTab] = useState('summary');
  const [confirmAction, setConfirmAction] = useState<'delete' | 'run' | 'stop' | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  const firstEditableRef = useRef<HTMLElement>(null);
  useEffect(() => {
    if (activeTab === 'settings' && canEdit && firstEditableRef.current) {
      const input = firstEditableRef.current.querySelector<HTMLInputElement>('input');
      if (input) input.focus();
    }
  }, [activeTab, canEdit]);

  // Use the unified form hook - pass taskId from route when task not yet loaded
  const {
    form,
    task,
    selectedTaskType,
  } = useTasksForm({
    taskId: initialTask?.id ?? taskIdFromRoute ?? undefined,
    task: initialTask || undefined,
    onCancel,
    createTask: async (data, startTime) => {
      const result = await createTask(data, startTime);
      if (onSaveProp) await onSaveProp(data);
      return result;
    },
    updateTask: async (id, data, startTime) => {
      const result = await updateTask(id, data, startTime);
      if (onSaveProp) await onSaveProp(data);
      return result;
    },
    deleteTask: async (id) => {
      await deleteTask(id);
      if (onDeleteProp) await onDeleteProp();
    },
  });

  const formData = form.data as TaskFormData & { startTime?: string };

  // Use task from form machine when loaded; fall back to initialTask (e.g. when TasksPage
  // fetched it before machine load, or in tests where useForm is mocked)
  const baseTask = task ?? initialTask ?? null;

  // Live-execution snapshot, freshest AUTHORITATIVE source first (ASTRUM-53525):
  //   liveTask (latest poll) → initialTask (TasksPage's seed, fetched via the hardened
  //   transform) → baseTask (the form machine's own task).
  // The form machine's task is only a form seed; its status is incidental and can be
  // stale (e.g. WAITING right after a refresh while the task is actually running). So
  // for the live-derived display fields we prefer the authoritative live source, which
  // makes the badge/Stop reflect true server state IMMEDIATELY on load — not after the
  // first poll. The id guards stop a previous task's snapshot bleeding through.
  let liveSource: Task | null = baseTask;
  if (initialTask && baseTask && initialTask.id === baseTask.id) {
    liveSource = initialTask;
  }
  if (liveTask && baseTask && liveTask.id === baseTask.id) {
    liveSource = liveTask;
  }

  // Form-state protection: we overlay ONLY the live-derived fields (status,
  // lastRun/Result, nextRun, runnable/stoppable, description) and deliberately leave
  // every form-bound field (enabled, alertEmail, notificationCondition, properties,
  // schedule) sourced from the form seed. Because the overlay never touches form data,
  // an in-flight edit can never be clobbered by a poll.
  const displayTask: Task | null = baseTask && liveSource
    ? {
        ...baseTask,
        status: liveSource.status,
        statusDescription: liveSource.statusDescription,
        lastRun: liveSource.lastRun,
        lastRunResult: liveSource.lastRunResult,
        nextRun: liveSource.nextRun,
        runnable: liveSource.runnable,
        stoppable: liveSource.stoppable,
      }
    : null;

  // Manual-only tasks (e.g. Data Repair Plan) cannot be scheduled — hide the Schedule tab entirely
  // (parity with Classic, which hides the schedule fieldset). The persisted schedule stays Manual
  // because there is no UI to change it. Decide from the task's typeId (known immediately).
  const isManualOnly = isManualOnlyTaskType(displayTask?.typeId ?? selectedTaskType?.id);

  // Defensive: if the active tab is ever the (now-removed) Schedule tab for a manual-only task,
  // fall back to Summary so the user never lands on a blank pane after the type resolves.
  useEffect(() => {
    if (isManualOnly && activeTab === 'schedule') {
      setActiveTab('summary');
    }
  }, [isManualOnly, activeTab]);

  const handlePropertyChange = useCallback((fieldId: string, value: string) => {
    const currentProps = formData.properties || {};
    form.send({ type: 'UPDATE', name: 'properties', value: { ...currentProps, [fieldId]: value } });
  }, [form, formData.properties]);

  const handleScheduleChange = useCallback((data: ScheduleData) => {
    form.send({ type: 'SCHEDULE_CHANGE', value: data.schedule, data } as any);
  }, [form]);

  // Build schedule data from form data
  const scheduleData: ScheduleData = useMemo(() => ({
    schedule: formData.schedule || 'manual',
    startDate: formData.startDate ? new Date(formData.startDate as any) : null,
    startTime: formData.startTime || '00:00',
    recurringDays: formData.recurringDays || [],
    cronExpression: formData.cronExpression || '',
    timeZoneOffset: formData.timeZoneOffset || '',
  }), [formData]);

  const allowedSchedules: ScheduleType[] | undefined = useMemo(
    () => (selectedTaskType?.concurrentRun === false ? ['manual', 'once'] : undefined),
    [selectedTaskType?.concurrentRun],
  );

  if (initialLoading || !displayTask) {
    return (
      <Box className="task-detail task-detail--loading" data-testid="task-detail-skeleton">
        <Flex direction="column" gap="3" p="4">
          <Skeleton width="40%" height="28px" />
          <Skeleton width="60%" height="18px" />
          <Skeleton width="100%" height="18px" />
          <Skeleton width="100%" height="18px" />
          <Skeleton width="80%" height="18px" />
          <Skeleton width="100%" height="18px" />
        </Flex>
      </Box>
    );
  }

  const footerActions = (
    <Flex gap="2" align="center">
      {activeTab === 'summary' && canEdit && !form.isDirty && displayTask && (
        <SettingsButton
          variant="secondary"
          icon={Edit3}
          onClick={() => setActiveTab('settings')}
          testId="task-edit-button"
        >
          {CommonStrings.ACTIONS.EDIT}
        </SettingsButton>
      )}
      {canDelete && (
        <SettingsButton
          variant="danger"
          icon={Trash2}
          onClick={() => {
            setConfirmAction('delete');
          }}
          disabled={form.isSaving}
          testId="form-delete"
          data-analytics-id="as-task-delete"
        >
          {CommonStrings.ACTIONS.DELETE}
        </SettingsButton>
      )}
      {displayTask.status === 'RUNNING' && canStop ? (
        <SettingsButton
          variant="secondary"
          icon={Square}
          onClick={() => setConfirmAction('stop')}
          testId="task-stop"
          data-analytics-id="as-task-stop"
        >
          {TasksStrings.DETAIL.STOP}
        </SettingsButton>
      ) : displayTask.enabled && canRun && displayTask.status !== 'RUNNING' ? (
        <SettingsButton
          variant="secondary"
          icon={Play}
          onClick={() => setConfirmAction('run')}
          testId="task-run"
          data-analytics-id="as-task-run"
        >
          {TasksStrings.DETAIL.RUN_NOW}
        </SettingsButton>
      ) : null}
    </Flex>
  );

  return (
    <Box className="task-detail" data-testid="task-detail">
      <SettingsForm
        onSubmit={() => form.send('SUBMIT')}
        onCancel={onCancel}
        loading={form.isSaving}
        pristine={form.isPristine}
        // Visually disable Save when validation fails (e.g. required Repository cleared).
        // Without this the canSave guard still blocks SUBMIT but the button looks enabled.
        submitDisabled={form.hasValidationErrors}
        error={externalError || form.saveError || undefined}
        submitLabel={CommonStrings.ACTIONS.SAVE}
        cancelLabel={CommonStrings.ACTIONS.BACK}
        showActions={canEdit}
        footerExtra={footerActions}
        testId="task-detail-form"
      >
        {/* Task Header */}
        <Box className="task-detail__header">
          <Flex justify="between" align="start">
            <Box>
              <Heading size="5" weight="bold">{displayTask.name}</Heading>
              <Text size="2" color="gray">{displayTask.typeName}</Text>
            </Box>
            <Flex gap="2" align="center">
              <Flex align="center" gap="2" className={`task-detail__status-badge ${displayTask.status === 'RUNNING' ? 'task-detail__status-badge--running' : ''}`}>
                {getStatusIcon(displayTask.status)}
                <Badge
                  color={getStatusBadgeColor(displayTask.status)}
                  variant="soft"
                  data-testid="task-status-badge"
                >
                  {displayTask.status}
                </Badge>
              </Flex>
              <Badge
                color={displayTask.enabled ? 'green' : 'gray'}
                variant="outline"
                data-testid="task-enabled-badge"
              >
                {displayTask.enabled ? TasksStrings.DETAIL.ENABLED : TasksStrings.DETAIL.DISABLED}
              </Badge>
            </Flex>
          </Flex>
        </Box>

        {/* 4 Tabs */}
        <Tabs.Root value={activeTab} onValueChange={setActiveTab} className="task-detail__tabs">
          <Tabs.List>
            <Tabs.Trigger value="summary">
              <FileText size={16} /> {TasksStrings.DETAIL.SUMMARY}
            </Tabs.Trigger>
            <Tabs.Trigger value="settings">
              <Settings size={16} /> {CommonStrings.LABELS.SETTINGS}
            </Tabs.Trigger>
            {!isManualOnly && (
              <Tabs.Trigger value="schedule">
                <Calendar size={16} /> {TasksStrings.DETAIL.SCHEDULE}
              </Tabs.Trigger>
            )}
            <Tabs.Trigger value="history">
              <History size={16} /> {TasksStrings.DETAIL.HISTORY}
            </Tabs.Trigger>
          </Tabs.List>

          <Box className="task-detail__tab-content">
            {/* Summary Tab */}
            <Tabs.Content value="summary">
              <Box className="task-detail__summary-grid">
                <SummaryItem
                  icon={<Info size={16} />}
                  label={TasksStrings.DETAIL.ID}
                  value={displayTask.id}
                />
                <SummaryItem
                  icon={<CheckCircle size={16} />}
                  label={CommonStrings.LABELS.STATUS}
                  value={displayTask.status}
                />
                <SummaryItem
                  icon={<FileText size={16} />}
                  label={TasksStrings.DETAIL.LAST_RESULT}
                  value={displayTask.lastRunResult || '-'}
                />
                <SummaryItem
                  icon={<Calendar size={16} />}
                  label={TasksStrings.DETAIL.SCHEDULE}
                  value={displayTask.schedule || '-'}
                  onClick={canEdit && !isManualOnly ? () => setActiveTab('schedule') : undefined}
                />
                <SummaryItem
                  icon={<Clock size={16} />}
                  label={TasksStrings.DETAIL.NEXT_RUN}
                  value={displayTask.nextRun ? formatDate(displayTask.nextRun) : '-'}
                />
                <SummaryItem
                  icon={<History size={16} />}
                  label={TasksStrings.DETAIL.LAST_RUN}
                  value={displayTask.lastRun ? formatDate(displayTask.lastRun) : TasksStrings.DETAIL.NEVER}
                />
                <SummaryItem
                  icon={<Mail size={16} />}
                  label={TasksStrings.DETAIL.ALERT_EMAIL}
                  value={displayTask.alertEmail || TasksStrings.DETAIL.NOT_CONFIGURED}
                  onClick={canEdit ? () => setActiveTab('settings') : undefined}
                />
                {displayTask.statusDescription && (
                  <SummaryItem
                    icon={<Info size={16} />}
                    label={CommonStrings.LABELS.DESCRIPTION}
                    value={displayTask.statusDescription}
                  />
                )}
                {Object.entries(displayTask.properties || {}).filter(([k]) => !k.startsWith('.')).map(([k, v]) => (
                  <SummaryItem
                    key={k}
                    icon={<Puzzle size={16} />}
                    label={k}
                    value={v || '-'}
                    onClick={canEdit ? () => setActiveTab('settings') : undefined}
                  />
                ))}
              </Box>
            </Tabs.Content>

            {/* Settings Tab (editable via form hook) */}
            <Tabs.Content value="settings">
              <SettingsFormSection title={TasksStrings.DETAIL.BASIC_SETTINGS}>
                <SettingsToggle
                  name="enabled"
                  label={TasksStrings.DETAIL.TASK_STATUS}
                  checked={formData.enabled}
                  onChange={(checked) => form.send({ type: 'UPDATE', name: 'enabled', value: checked })}
                  disabled={!canEdit || form.isSaving}
                  helpText={TasksStrings.DETAIL.ENABLE_DISABLE_TASK}
                  checkedText={TasksStrings.DETAIL.ENABLED}
                  uncheckedText={TasksStrings.DETAIL.DISABLED}
                />
                <SettingsTextInput
                  name="name"
                  label={CommonStrings.LABELS.NAME}
                  value={displayTask.name}
                  onChange={() => {}}
                  disabled
                  helpText={TasksStrings.DETAIL.TASK_NAME_CANNOT_CHANGED}
                />
                <SettingsTextInput
                  name="typeId"
                  label={TasksStrings.DETAIL.TASK_TYPE}
                  value={displayTask.typeName}
                  onChange={() => {}}
                  disabled
                  helpText={TasksStrings.DETAIL.TASK_TYPE_CANNOT_CHANGED_AFTER}
                />
                <SettingsTextInput
                  {...form.field('alertEmail')}
                  label={TasksStrings.DETAIL.NOTIFICATION_EMAIL}
                  disabled={!canEdit || form.isSaving}
                  helpText={TasksStrings.DETAIL.LEAVE_EMPTY_DISABLE_NOTIFICATIONS_ENTER}
                  placeholder="admin@example.com"
                  inputRef={firstEditableRef}
                />
                {formData.alertEmail && (
                  <SettingsSelect
                    name="notificationCondition"
                    label={TasksStrings.DETAIL.SEND_NOTIFICATION}
                    value={formData.notificationCondition || 'FAILURE'}
                    onChange={(v) => form.send({ type: 'UPDATE', name: 'notificationCondition', value: v })}
                    disabled={!canEdit || form.isSaving}
                    options={NOTIFICATION_CONDITIONS}
                    helpText={TasksStrings.DETAIL.WHEN_SEND_EMAIL_NOTIFICATIONS}
                  />
                )}
              </SettingsFormSection>

              {selectedTaskType?.formFields && selectedTaskType.formFields.length > 0 && (
                <SettingsFormSection title={TasksStrings.DETAIL.TASK_CONFIGURATION}>
                  <DynamicFormFields
                    taskType={selectedTaskType}
                    values={formData.properties}
                    onChange={handlePropertyChange}
                    errors={form.validationErrors?.properties as any}
                    disabled={!canEdit || form.isSaving}
                  />
                </SettingsFormSection>
              )}
            </Tabs.Content>

            {/* Schedule Tab — omitted for manual-only tasks (no configurable schedule) */}
            {!isManualOnly && (
              <Tabs.Content value="schedule">
                <SettingsFormSection title={TasksStrings.DETAIL.SCHEDULE_CONFIGURATION}>
                  <TaskScheduler
                    value={scheduleData}
                    onChange={handleScheduleChange}
                    errors={{
                      schedule: form.touched?.schedule ? form.validationErrors?.schedule : undefined,
                      startDate: form.touched?.startDate ? form.validationErrors?.startDate : undefined,
                      startTime: form.touched?.startTime ? form.validationErrors?.startTime : undefined,
                      recurringDays: form.touched?.recurringDays ? form.validationErrors?.recurringDays : undefined,
                      cronExpression: form.touched?.cronExpression ? form.validationErrors?.cronExpression : undefined,
                    }}
                    disabled={!canEdit || form.isSaving}
                    allowedSchedules={allowedSchedules}
                  />
                </SettingsFormSection>
              </Tabs.Content>
            )}

            {/* History Tab */}
            <Tabs.Content value="history">
              <TaskHistory task={displayTask} />
            </Tabs.Content>
          </Box>
        </Tabs.Root>
      </SettingsForm>

      <DeleteConfirmationModal
        open={confirmAction === 'delete'}
        onClose={() => setConfirmAction(null)}
        onConfirm={async () => {
          setIsDeleting(true);
          try {
            setConfirmAction(null);
            form.send('DELETE');
            await form.confirmDelete();
          } finally {
            setIsDeleting(false);
          }
        }}
        entityName={displayTask.name}
        entityType="task"
        loading={isDeleting}
      />
      <ConfirmDialog
        open={confirmAction === 'run'}
        testId="task-run-dialog"
        onOpenChange={(open) => { if (!open) setConfirmAction(null); }}
        title={TasksStrings.DETAIL.RUN_TASK}
        message={TasksStrings.DETAIL.RUN_CONFIRM(displayTask.name)}
        confirmLabel={TasksStrings.DETAIL.RUN}
        variant="warning"
        onConfirm={() => { setConfirmAction(null); onRun(); }}
      />
      <ConfirmDialog
        open={confirmAction === 'stop'}
        testId="task-stop-dialog"
        onOpenChange={(open) => { if (!open) setConfirmAction(null); }}
        title={TasksStrings.DETAIL.STOP_TASK}
        message={TasksStrings.DETAIL.STOP_CONFIRM(displayTask.name)}
        confirmLabel={TasksStrings.DETAIL.STOP}
        variant="warning"
        onConfirm={() => { setConfirmAction(null); onStop(); }}
      />
    </Box>
  );
}

function SummaryItem({
  icon,
  label,
  value,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  onClick?: () => void;
}) {
  return (
    <Box
      className={`task-detail__summary-item ${onClick ? 'task-detail__summary-item--clickable' : ''}`}
      onClick={onClick}
    >
      <Flex align="center" gap="2">
        <Box className="task-detail__summary-icon">{icon}</Box>
        <Text size="1" color="gray" className="task-detail__summary-label">{label}:</Text>
      </Flex>
      <Text size="2" className="task-detail__summary-value">{value}</Text>
    </Box>
  );
}

export default TaskDetail;
