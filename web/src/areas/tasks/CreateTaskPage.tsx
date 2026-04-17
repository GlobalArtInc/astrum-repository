import { Button, Flex, Spinner, Table, Text, TextField } from '@radix-ui/themes';
import { ArrowLeft, ArrowUp, ChevronRight, Search, Settings } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { ApiError, api, fieldName, useApi, useInvalidateApi } from '../../api';
import { useT } from '../../i18n';
import { describeError, ErrorState, FormError, Page, UnsavedChangesGuard, useToast } from '../../ui';
import { SettingsFields, SettingsForm, SettingsSection } from '../../ui/SettingsForm';
import { categoryLabel, typeDescription } from './components/bits';
import { ScheduleFields } from './components/ScheduleFields';
import { BasicSettings, fieldLabel, PropertyFields } from './components/TaskFields';
import {
  CHOICES,
  type Choices,
  emptyTask,
  LIST_PAGE,
  missingRequired,
  type Problems,
  TASKS,
  type TaskValues,
  type Template,
  TYPES,
  taskHref,
  toRequest,
  UI_TASKS,
  validateSchedule,
  validateSettings,
} from './model';
import styles from './Tasks.module.css';
import { useProblemTexts } from './texts';

const SCHEDULE_FIELDS = new Set([
  'schedule',
  'startDate',
  'startTime',
  'recurringDays',
  'cronExpression',
  'frequency',
  'timeZoneOffset',
]);

function TypePicker({
  templates,
  chosen,
  onChoose,
}: {
  templates: Template[];
  chosen: string | null;
  onChoose(template: Template): void;
}) {
  const t = useT();
  const [filter, setFilter] = useState('');
  const shown = useMemo(() => {
    const words = filter.trim().toLowerCase().split(/\s+/).filter(Boolean);
    return templates
      .filter((template) => {
        const text =
          `${template.name} ${categoryLabel(t, template.type)} ${typeDescription(t, template.type)}`.toLowerCase();
        return words.every((word) => text.includes(word));
      })
      .sort((left, right) => left.name.localeCompare(right.name));
  }, [templates, filter, t]);
  return (
    <div className={styles.typePicker}>
      <div className={styles.typeSearch}>
        <TextField.Root
          className={styles.search}
          type="search"
          value={filter}
          placeholder={t('tasks.create.filterTypes')}
          aria-label={t('tasks.create.filterTypes')}
          onChange={(event) => setFilter(event.target.value)}
        >
          <TextField.Slot>
            <Search size={16} aria-hidden="true" />
          </TextField.Slot>
        </TextField.Root>
        <Text size="2" color="gray">
          {t('tasks.create.typeCount', { count: shown.length })}
        </Text>
      </div>
      <Table.Root variant="ghost" className={styles.typeTable} aria-label={t('tasks.create.types')}>
        <Table.Header>
          <Table.Row>
            <Table.ColumnHeaderCell aria-sort="ascending">
              <span className={styles.sortHead}>
                {t('tasks.create.typeName')}
                <ArrowUp size={14} aria-hidden="true" />
              </span>
            </Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell>{t('tasks.create.category')}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell className={styles.typeDescription}>
              {t('tasks.create.typeDescription')}
            </Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell className={styles.arrowCell} />
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {shown.map((template) => (
            <Table.Row
              key={template.type}
              className={styles.typeRow}
              data-selected={chosen === template.type}
              onClick={() => onChoose(template)}
            >
              <Table.Cell>
                <button
                  type="button"
                  className={styles.typeName}
                  aria-pressed={chosen === template.type}
                  onClick={(event) => {
                    event.stopPropagation();
                    onChoose(template);
                  }}
                >
                  {template.name}
                </button>
              </Table.Cell>
              <Table.Cell>{categoryLabel(t, template.type)}</Table.Cell>
              <Table.Cell className={`${styles.muted} ${styles.typeDescription}`}>
                {typeDescription(t, template.type)}
              </Table.Cell>
              <Table.Cell className={styles.arrowCell}>
                <ChevronRight size={16} aria-hidden="true" />
              </Table.Cell>
            </Table.Row>
          ))}
          {shown.length === 0 && (
            <Table.Row>
              <Table.Cell colSpan={4}>
                <Text color="gray">{t('tasks.create.noTypes')}</Text>
              </Table.Cell>
            </Table.Row>
          )}
        </Table.Body>
      </Table.Root>
    </div>
  );
}

export default function CreateTaskPage() {
  const t = useT();
  const texts = useProblemTexts();
  const toast = useToast();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const params = useParams();
  const templates = useApi<Template[]>(TYPES);
  const choices = useApi<Choices>(CHOICES);
  const [template, setTemplate] = useState<Template | null>(null);
  const [step, setStep] = useState(0);
  const [values, setValues] = useState<TaskValues | null>(null);
  const [errors, setErrors] = useState<Problems>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [leaving, setLeaving] = useState<string | null>(null);
  useEffect(() => {
    if (leaving !== null) {
      navigate(leaving);
    }
  }, [leaving, navigate]);
  const creatable = useMemo(
    () => templates.data?.filter((found) => found.creatable !== false) ?? [],
    [templates.data],
  );

  const choose = (chosen: Template) => {
    if (template?.type !== chosen.type) {
      setTemplate(chosen);
      setValues(emptyTask(chosen));
      setErrors({});
    }
  };

  useEffect(() => {
    if (params.type && templates.data && !template) {
      const found = templates.data.find(
        (candidate) => candidate.type === params.type && candidate.creatable !== false,
      );
      if (found) {
        setTemplate(found);
        setValues(emptyTask(found));
        setStep(1);
      }
    }
  }, [params.type, templates.data, template]);

  const set = <K extends keyof TaskValues>(name: K, value: TaskValues[K]) => {
    setValues((current) => (current ? { ...current, [name]: value } : current));
    setErrors((current) => {
      const next = { ...current };
      delete next[name as string];
      return next;
    });
  };

  const next = async () => {
    if (!values || !template) {
      return;
    }
    if (step === 0) {
      setStep(1);
      return;
    }
    if (step === 1) {
      const problems = validateSettings(values, template, texts, (field) => fieldLabel(t, field));
      setErrors(problems);
      if (Object.keys(problems).length === 0) {
        setStep(2);
      }
      return;
    }
    const problems = validateSchedule(values.schedule, texts);
    setErrors(problems);
    if (Object.keys(problems).length > 0) {
      return;
    }
    setBusy(true);
    setFormError(null);
    try {
      const created = await api.post<{ id: string; name: string }>(TASKS, toRequest(values, template.type));
      toast.success(t('tasks.toasts.created', { name: created.name }));
      await Promise.all([invalidate(UI_TASKS), invalidate(TASKS)]);
      setLeaving(taskHref(created.id));
    } catch (error) {
      if (error instanceof ApiError && error.fieldErrors.length > 0) {
        const mapped: Problems = {};
        let scheduleProblem = false;
        for (const problem of error.fieldErrors) {
          const name = fieldName(problem.id).split('.').pop() ?? problem.id;
          mapped[name] = problem.message;
          scheduleProblem ||= SCHEDULE_FIELDS.has(name);
        }
        setErrors(mapped);
        setStep(scheduleProblem ? 2 : 1);
      } else {
        setFormError(describeError(t, error));
      }
    } finally {
      setBusy(false);
    }
  };

  const titles = [t('tasks.create.selectType'), t('tasks.create.configure'), t('tasks.create.schedule')];

  return (
    <Page
      className={styles.page}
      title={t('tasks.create.title')}
      breadcrumbs={[
        { label: t('tasks.crumbs.settings'), to: '/settings' },
        { label: t('tasks.list.title'), to: LIST_PAGE },
        { label: t('tasks.create.crumb') },
      ]}
    >
      <SettingsForm
        label={t('tasks.create.title')}
        onSubmit={() => void next()}
        progress={{ step: step + 1, steps: 3, label: titles[step] }}
        start={
          <Button asChild variant="outline">
            <Link to={LIST_PAGE}>{t('tasks.actions.cancel')}</Link>
          </Button>
        }
        end={
          <>
            {step > 0 && (
              <Button type="button" variant="soft" onClick={() => setStep(step - 1)} disabled={busy}>
                <ArrowLeft size={16} aria-hidden="true" />
                {t('tasks.actions.back')}
              </Button>
            )}
            <Button
              type="submit"
              disabled={
                template === null ||
                (step === 1 && !!values && values.name.trim() === '') ||
                (step === 1 && !!values && missingRequired(template, values))
              }
              loading={busy}
            >
              {step === 2 ? t('tasks.actions.create') : t('tasks.actions.continue')}
            </Button>
          </>
        }
      >
        <UnsavedChangesGuard when={values !== null && leaving === null} />
        <h2 className={styles.stepTitle}>{titles[step]}</h2>
        {step === 0 &&
          (templates.isPending ? (
            <Flex justify="center" py="8">
              <Spinner size="3" />
            </Flex>
          ) : templates.error ? (
            <ErrorState error={templates.error} onRetry={() => void templates.refetch()} />
          ) : (
            <TypePicker templates={creatable} chosen={template?.type ?? null} onChoose={choose} />
          ))}
        {step > 0 && template && values && (
          <>
            {step === 1 && (
              <>
                <div className={styles.typeCard}>
                  <Settings size={24} aria-hidden="true" />
                  <div>
                    <div className={styles.typeCardTitle}>{template.name}</div>
                    <div className={styles.typeCardText}>
                      {categoryLabel(t, template.type)} • {typeDescription(t, template.type)}
                    </div>
                  </div>
                </div>
                <SettingsSection title={t('tasks.settings.title')}>
                  <SettingsFields>
                    <BasicSettings values={values} set={set} errors={errors} />
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
                        choices={choices.data}
                      />
                    </SettingsFields>
                  </SettingsSection>
                )}
              </>
            )}
            {step === 2 && (
              <SettingsFields>
                <ScheduleFields
                  values={values.schedule}
                  onChange={(schedule) => set('schedule', schedule)}
                  errors={errors}
                />
              </SettingsFields>
            )}
            <FormError message={formError} />
          </>
        )}
      </SettingsForm>
    </Page>
  );
}
