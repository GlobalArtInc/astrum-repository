import { Button, Flex, Spinner, Text } from '@radix-ui/themes';
import {
  CalendarDays,
  CircleAlert,
  Clock,
  FileText,
  History,
  Info,
  Mail,
  Pencil,
  Play,
  Settings,
  Square,
  Trash2,
} from 'lucide-react';
import { type ReactNode, useEffect, useRef, useState } from 'react';
import { Link, useNavigate, useParams, useSearchParams } from 'react-router';
import { ApiError, api, fieldName, path, useApi, useInvalidateApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import {
  ConfirmDialog,
  describeError,
  ErrorState,
  FormError,
  Page,
  QueryTabs,
  UnsavedChangesGuard,
  useToast,
} from '../../ui';
import { TAB_PARAM } from '../../ui/QueryTabs';
import { SettingsFields, SettingsForm, SettingsSection, settingsButton } from '../../ui/SettingsForm';
import {
  resultLabel,
  scheduleLabel,
  statusLabel,
  TitleStateBadge,
  typeDescription,
  useLocalTime,
} from './components/bits';
import { DeleteTaskDialog } from './components/DeleteTaskDialog';
import { RunHistory } from './components/RunHistory';
import { ScheduleFields } from './components/ScheduleFields';
import { BasicSettings, fieldLabel, PropertyFields } from './components/TaskFields';
import {
  CHOICES,
  type Choices,
  DELETE,
  fromTask,
  isActive,
  LIST_PAGE,
  type Problems,
  START,
  STOP,
  scheduleKind,
  statusOf,
  TASKS,
  type Task,
  type TaskValues,
  toRequest,
  UI_TASKS,
  UPDATE,
  validateSchedule,
  validateSettings,
} from './model';
import styles from './Tasks.module.css';
import { useTaskActions } from './TasksPage';
import { useProblemTexts } from './texts';

function Term({
  icon,
  label,
  children,
  wide,
  accent,
}: {
  icon: ReactNode;
  label: string;
  children: ReactNode;
  wide?: boolean;
  accent?: boolean;
}) {
  return (
    <div className={wide ? styles.wide : undefined}>
      <div className={accent ? `${styles.term} ${styles.accentTerm}` : styles.term}>
        {icon}
        <span>{label}</span>
      </div>
      <div className={styles.value}>{children}</div>
    </div>
  );
}

function Summary({ task, onOpen }: { task: Task; onOpen(tab: string): void }) {
  const t = useT();
  const time = useLocalTime();
  const kind = scheduleKind(task.frequency);
  const started = task.current?.startedAt ?? task.lastRun?.startedAt ?? null;
  return (
    <div className={styles.summary}>
      <Term icon={<Info size={16} aria-hidden="true" />} label={t('tasks.summary.id')}>
        {task.id}
      </Term>
      <Term icon={<FileText size={16} aria-hidden="true" />} label={t('tasks.summary.lastResult')}>
        {task.lastRun ? resultLabel(t, task.lastRun.state) : t('tasks.summary.none')}
      </Term>
      <Term icon={<CalendarDays size={16} aria-hidden="true" />} label={t('tasks.summary.schedule')} accent>
        <button type="button" className={styles.valueLink} onClick={() => onOpen('schedule')}>
          {scheduleLabel(t, kind)}
        </button>
      </Term>
      <Term icon={<Clock size={16} aria-hidden="true" />} label={t('tasks.summary.nextRun')}>
        {task.nextRun ? time(task.nextRun) : t('tasks.summary.none')}
      </Term>
      <Term icon={<History size={16} aria-hidden="true" />} label={t('tasks.summary.lastRun')}>
        {started ? time(started) : t('tasks.summary.never')}
      </Term>
      <Term icon={<Mail size={16} aria-hidden="true" />} label={t('tasks.summary.alertEmail')} accent>
        <button type="button" className={styles.valueLink} onClick={() => onOpen('settings')}>
          {task.alertEmail ?? t('tasks.summary.notConfigured')}
        </button>
      </Term>
      <Term icon={<Info size={16} aria-hidden="true" />} label={t('tasks.summary.description')} wide>
        {typeDescription(t, task.type)}
      </Term>
    </div>
  );
}

function TaskView({ task, choices }: { task: Task; choices: Choices | undefined }) {
  const t = useT();
  const texts = useProblemTexts();
  const toast = useToast();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const actions = useTaskActions();
  const [params, setParams] = useSearchParams();
  const canUpdate = useCan(UPDATE);
  const canDelete = useCan(DELETE);
  const canStart = useCan(START);
  const canStop = useCan(STOP);
  const template = task.template;
  const [values, setValues] = useState<TaskValues | null>(template ? fromTask(task, template) : null);
  const [baseline, setBaseline] = useState(values);
  const [errors, setErrors] = useState<Problems>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [confirm, setConfirm] = useState<'run' | 'stop' | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [leaving, setLeaving] = useState<string | null>(null);
  useEffect(() => {
    if (leaving !== null) {
      navigate(leaving);
    }
  }, [leaving, navigate]);
  const dirty = JSON.stringify(values) !== JSON.stringify(baseline);
  const dirtyRef = useRef(dirty);
  dirtyRef.current = dirty;
  const editable = canUpdate && task.known;

  useEffect(() => {
    if (template && !dirtyRef.current) {
      const fresh = fromTask(task, template);
      setValues(fresh);
      setBaseline(fresh);
    }
  }, [task, template]);

  const set = <K extends keyof TaskValues>(name: K, value: TaskValues[K]) => {
    setValues((current) => (current ? { ...current, [name]: value } : current));
  };

  const save = async () => {
    if (!values || !template || !dirty) {
      return;
    }
    const problems = {
      ...validateSettings(values, template, texts, (field) => fieldLabel(t, field)),
      ...validateSchedule(values.schedule, texts),
    };
    setErrors(problems);
    if (Object.keys(problems).length > 0) {
      return;
    }
    setSaving(true);
    setFormError(null);
    try {
      await api.put(`${TASKS}${path`/${task.id}`}`, toRequest(values));
      toast.success(t('tasks.toasts.updated', { name: task.name }));
      setBaseline(values);
      dirtyRef.current = false;
      await Promise.all([invalidate(UI_TASKS), invalidate(TASKS)]);
    } catch (error) {
      if (error instanceof ApiError && error.fieldErrors.length > 0) {
        const mapped: Problems = {};
        for (const problem of error.fieldErrors) {
          mapped[fieldName(problem.id).split('.').pop() ?? problem.id] = problem.message;
        }
        setErrors(mapped);
        setFormError(error.fieldErrors.map((problem) => problem.message).join(' '));
      } else {
        setFormError(describeError(t, error));
      }
    } finally {
      setSaving(false);
    }
  };

  const active = isActive(task);
  const status = statusOf(task);
  const tab = params.get(TAB_PARAM) ?? 'summary';
  const openTab = (value: string) =>
    setParams(
      (previous) => {
        const next = new URLSearchParams(previous);
        next.set(TAB_PARAM, value);
        return next;
      },
      { replace: true },
    );
  const openSettings = () => openTab('settings');

  return (
    <Page
      className={styles.gutter}
      title={task.name}
      titleExtra={<TitleStateBadge status={status} label={statusLabel(t, status)} />}
      description={task.typeName}
      breadcrumbs={[
        { label: t('tasks.crumbs.settings'), to: '/settings' },
        { label: t('tasks.list.title'), to: LIST_PAGE },
        { label: task.name },
      ]}
    >
      <SettingsForm
        wide
        label={task.name}
        onSubmit={() => void save()}
        unsaved={dirty && editable ? t('tasks.detail.unsaved') : null}
        start={
          <>
            {tab === 'summary' && editable && (
              <Button
                type="button"
                size="3"
                variant="outline"
                color="gray"
                className={settingsButton}
                onClick={openSettings}
              >
                <Pencil size={16} aria-hidden="true" />
                {t('tasks.actions.edit')}
              </Button>
            )}
            {canDelete && (
              <Button
                type="button"
                size="3"
                color="red"
                className={settingsButton}
                onClick={() => setDeleting(true)}
              >
                <Trash2 size={16} aria-hidden="true" />
                {t('tasks.actions.delete')}
              </Button>
            )}
            {active
              ? canStop && (
                  <Button
                    type="button"
                    size="3"
                    variant="outline"
                    color="gray"
                    className={settingsButton}
                    onClick={() => setConfirm('stop')}
                  >
                    <Square size={16} aria-hidden="true" />
                    {t('tasks.actions.stop')}
                  </Button>
                )
              : canStart && (
                  <Button
                    type="button"
                    size="3"
                    variant="outline"
                    color="gray"
                    className={settingsButton}
                    onClick={() => setConfirm('run')}
                    disabled={!task.enabled || !task.known}
                  >
                    <Play size={16} aria-hidden="true" />
                    {t('tasks.actions.runNow')}
                  </Button>
                )}
          </>
        }
        end={
          <>
            <Button asChild variant="outline">
              <Link to={LIST_PAGE}>{t('tasks.actions.back')}</Link>
            </Button>
            {editable && (
              <Button type="submit" disabled={!dirty} loading={saving}>
                {t('tasks.actions.save')}
              </Button>
            )}
          </>
        }
      >
        <UnsavedChangesGuard when={editable && dirty && leaving === null} />
        {!task.known && (
          <Flex gap="2" align="center" mb="4">
            <CircleAlert size={16} color="var(--amber-11)" aria-hidden="true" />
            <Text size="2" color="amber">
              {t('tasks.detail.unknownType')}
            </Text>
          </Flex>
        )}
        <FormError message={formError} />
        <QueryTabs
          className={styles.tabs}
          tabs={[
            {
              value: 'summary',
              label: (
                <Flex gap="2" align="center">
                  <FileText size={16} aria-hidden="true" />
                  {t('tasks.detail.summary')}
                </Flex>
              ),
              content: <Summary task={task} onOpen={openTab} />,
            },
            {
              value: 'settings',
              label: (
                <Flex gap="2" align="center">
                  <Settings size={16} aria-hidden="true" />
                  {t('tasks.detail.settings')}
                </Flex>
              ),
              content:
                values && template ? (
                  <>
                    <SettingsSection title={t('tasks.settings.basic')}>
                      <SettingsFields>
                        <BasicSettings
                          values={values}
                          set={set}
                          errors={errors}
                          disabled={!editable}
                          existing
                          typeName={task.typeName}
                        />
                      </SettingsFields>
                    </SettingsSection>
                    {template.formFields.length > 0 && (
                      <SettingsSection title={t('tasks.settings.configuration')}>
                        <SettingsFields>
                          <PropertyFields
                            template={template}
                            values={values.properties}
                            onChange={(properties) => set('properties', properties)}
                            errors={errors}
                            choices={choices}
                            disabled={!editable}
                          />
                        </SettingsFields>
                      </SettingsSection>
                    )}
                  </>
                ) : (
                  <Text color="gray">{t('tasks.detail.unknownType')}</Text>
                ),
            },
            {
              value: 'schedule',
              label: (
                <Flex gap="2" align="center">
                  <CalendarDays size={16} aria-hidden="true" />
                  {t('tasks.detail.schedule')}
                </Flex>
              ),
              content: values ? (
                <SettingsSection title={t('tasks.schedule.title')}>
                  <SettingsFields>
                    <ScheduleFields
                      values={values.schedule}
                      onChange={(schedule) => set('schedule', schedule)}
                      errors={errors}
                      disabled={!editable}
                    />
                  </SettingsFields>
                </SettingsSection>
              ) : null,
            },
            {
              value: 'history',
              label: (
                <Flex gap="2" align="center">
                  <History size={16} aria-hidden="true" />
                  {t('tasks.detail.history')}
                </Flex>
              ),
              content: <RunHistory task={task} />,
            },
          ]}
        />
      </SettingsForm>
      <ConfirmDialog
        open={confirm === 'run'}
        onOpenChange={(open) => !open && setConfirm(null)}
        onConfirm={() => actions.run(task)}
        title={t('tasks.run.title')}
        description={t('tasks.run.text', { name: task.name })}
        confirmLabel={t('tasks.run.confirm')}
      />
      <ConfirmDialog
        open={confirm === 'stop'}
        onOpenChange={(open) => !open && setConfirm(null)}
        onConfirm={() => actions.stop(task)}
        title={t('tasks.stop.title')}
        description={t('tasks.stop.text', { name: task.name })}
        confirmLabel={t('tasks.stop.confirm')}
        danger
      />
      <DeleteTaskDialog
        open={deleting}
        onOpenChange={setDeleting}
        name={task.name}
        onDelete={async () => {
          await actions.remove(task);
          setLeaving(LIST_PAGE);
        }}
      />
    </Page>
  );
}

export default function TaskPage() {
  const t = useT();
  const { id = '' } = useParams();
  const task = useApi<Task>(`${UI_TASKS}${path`/${id}`}`, {
    refetchInterval: (query) => (query.state.data && isActive(query.state.data) ? 2000 : false),
  });
  const choices = useApi<Choices>(CHOICES);
  if (task.isPending) {
    return (
      <Page
        className={styles.gutter}
        title={t('tasks.detail.loading')}
        breadcrumbs={[
          { label: t('tasks.crumbs.settings'), to: '/settings' },
          { label: t('tasks.list.title'), to: LIST_PAGE },
        ]}
      >
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      </Page>
    );
  }
  if (task.error) {
    return (
      <Page
        className={styles.gutter}
        title={task.error.status === 404 ? t('tasks.detail.notFound') : t('tasks.detail.failed')}
        breadcrumbs={[
          { label: t('tasks.crumbs.settings'), to: '/settings' },
          { label: t('tasks.list.title'), to: LIST_PAGE },
        ]}
      >
        <ErrorState error={task.error} onRetry={() => void task.refetch()} />
      </Page>
    );
  }
  return <TaskView task={task.data} choices={choices.data} />;
}
