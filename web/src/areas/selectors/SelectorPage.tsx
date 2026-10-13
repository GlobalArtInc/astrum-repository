import { Button, Flex, Select, Spinner, TextArea } from '@radix-ui/themes';
import { useQuery } from '@tanstack/react-query';
import {
  Check,
  ChevronDown,
  ChevronRight,
  Eye,
  Info,
  Link2,
  Plus,
  Trash2,
  TriangleAlert,
} from 'lucide-react';
import { type FormEvent, useId, useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { type ApiError, api, path, useApi, useInvalidateApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import { ConfirmDialog, ErrorState, FormError, Page, UnsavedChangesGuard, useToast } from '../../ui';
import { Input } from '../identity/shared/Fields';
import type { Privilege } from '../security/api';
import { SearchBox } from '../security/shared/AdminList';
import { EditBar, EditForm, FormSection, ReadOnlyValue } from '../security/shared/Edit';
import editStyles from '../security/shared/Edit.module.css';
import { useLeave } from '../security/shared/useLeave';
import { explainExpression } from './explain';
import {
  type ContentSelector,
  EMPTY_SELECTOR,
  fromSelector,
  LIST_PAGE,
  PREVIEW,
  PRIVILEGES,
  type Preview,
  REPOSITORIES,
  type Repository,
  SELECTORS,
  type SelectorValues,
  toCreateRequest,
  toUpdateRequest,
  validateSelector,
} from './model';
import { usePreview } from './preview';
import styles from './Selectors.module.css';

const EXAMPLES = [
  'format == "raw"',
  'format == "maven2"',
  'format == "npm"',
  'format == "docker"',
  'path =^ "/org"',
  'format == "maven2" and path =^ "/org"',
];

const FORMAT_PREFIX = 'format:';

function Reference() {
  const t = useT();
  const [open, setOpen] = useState(false);
  return (
    <section className={styles.reference}>
      <button
        type="button"
        className={styles.referenceToggle}
        aria-expanded={open}
        onClick={() => setOpen(!open)}
      >
        {open ? <ChevronDown size={16} aria-hidden="true" /> : <ChevronRight size={16} aria-hidden="true" />}
        {t('selectors.reference.title')}
      </button>
      {open && (
        <div className={styles.referenceBody}>
          <h4>{t('selectors.reference.attributes')}</h4>
          <table className={styles.referenceTable}>
            <thead>
              <tr>
                <th>{t('selectors.reference.attribute')}</th>
                <th>{t('selectors.reference.description')}</th>
                <th>{t('selectors.reference.operators')}</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>
                  <code>format</code>
                </td>
                <td>{t('selectors.reference.format')}</td>
                <td>
                  <code>==, !=, =~</code>
                </td>
              </tr>
              <tr>
                <td>
                  <code>path</code>
                </td>
                <td>{t('selectors.reference.path')}</td>
                <td>
                  <code>==, !=, =~, =^</code>
                </td>
              </tr>
            </tbody>
          </table>
          <h4>{t('selectors.reference.comparison')}</h4>
          <table className={styles.referenceTable}>
            <thead>
              <tr>
                <th>{t('selectors.reference.operator')}</th>
                <th>{t('selectors.reference.description')}</th>
              </tr>
            </thead>
            <tbody>
              {(
                [
                  ['==', 'selectors.csel.equals'],
                  ['!=', 'selectors.csel.notEquals'],
                  ['=~', 'selectors.reference.matches'],
                  ['=^', 'selectors.csel.startsWith'],
                ] as const
              ).map(([operator, key]) => (
                <tr key={operator}>
                  <td>
                    <code>{operator}</code>
                  </td>
                  <td>{t(key)}</td>
                </tr>
              ))}
            </tbody>
          </table>
          <h4>{t('selectors.reference.logical')}</h4>
          <table className={styles.referenceTable}>
            <thead>
              <tr>
                <th>{t('selectors.reference.operator')}</th>
                <th>{t('selectors.reference.description')}</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>
                  <code>and</code>
                </td>
                <td>{t('selectors.reference.and')}</td>
              </tr>
              <tr>
                <td>
                  <code>or</code>
                </td>
                <td>{t('selectors.reference.or')}</td>
              </tr>
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}

function Results({ expression }: { expression: string }) {
  const t = useT();
  const id = useId();
  const repositories = useApi<Repository[]>(REPOSITORIES);
  const [target, setTarget] = useState('');
  const [asked, setAsked] = useState<string | null>(null);
  const [filter, setFilter] = useState('');
  const formats = [...new Set((repositories.data ?? []).map((repository) => repository.format))].sort();
  const result = useQuery<Preview, ApiError>({
    queryKey: ['selectors', 'results', expression, asked],
    queryFn: ({ signal }) => {
      const body =
        asked === '*'
          ? { expression, repository: '*' }
          : asked?.startsWith(FORMAT_PREFIX)
            ? { expression, format: asked.slice(FORMAT_PREFIX.length) }
            : { expression, repository: asked };
      return api.post<Preview>(PREVIEW, body, { signal });
    },
    enabled: asked !== null && expression.trim() !== '',
    retry: false,
  });
  const words = filter.trim().toLowerCase();
  const rows = (result.data?.items ?? []).filter((item) => !words || item.path.toLowerCase().includes(words));
  return (
    <FormSection title={t('selectors.results.title')}>
      <p className={styles.intro}>{t('selectors.results.intro')}</p>
      <div className={styles.previewRow}>
        <div className={styles.previewPicker}>
          <label htmlFor={id} className={styles.previewLabel}>
            {t('selectors.preview.repositoryPicker')}
          </label>
          <Select.Root value={target || undefined} onValueChange={setTarget}>
            <Select.Trigger
              id={id}
              placeholder={t('selectors.results.choose')}
              className={styles.previewSelect}
            />
            <Select.Content position="popper">
              <Select.Item value="*">{t('selectors.results.all')}</Select.Item>
              {formats.map((format) => (
                <Select.Item key={format} value={`${FORMAT_PREFIX}${format}`}>
                  {t('selectors.results.allOf', { format })}
                </Select.Item>
              ))}
              {(repositories.data ?? []).map((repository) => (
                <Select.Item key={repository.name} value={repository.name}>
                  {repository.name}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        </div>
        <Button
          type="button"
          size="3"
          variant="outline"
          color="gray"
          className={styles.previewButton}
          disabled={target === '' || expression.trim() === ''}
          onClick={() => setAsked(target)}
        >
          {t('selectors.results.run')}
        </Button>
      </div>
      <div className={styles.resultsFilter}>
        <SearchBox value={filter} onChange={setFilter} placeholder={t('selectors.results.filter')} />
      </div>
      <table className={styles.results}>
        <thead>
          <tr>
            <th>{t('selectors.form.name')}</th>
          </tr>
        </thead>
        <tbody>
          {result.isFetching ? (
            <tr>
              <td className={styles.resultsEmpty}>
                <Spinner />
              </td>
            </tr>
          ) : rows.length === 0 ? (
            <tr>
              <td className={styles.resultsEmpty}>{t('selectors.results.none')}</td>
            </tr>
          ) : (
            rows.map((item) => (
              <tr key={`${item.repository}/${item.path}`}>
                <td>{item.path}</td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </FormSection>
  );
}

function SelectorForm({ selector }: { selector?: ContentSelector }) {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const { leaving, leave } = useLeave();
  const creating = selector === undefined;
  const canWrite = useCan(creating ? 'astrum:selectors:create' : 'astrum:selectors:update');
  const canDelete = useCan('astrum:selectors:delete');
  const canPrivileges = useCan('astrum:privileges:read');
  const readOnly = !canWrite;
  const initial = useMemo(() => (selector ? fromSelector(selector) : EMPTY_SELECTOR), [selector]);
  const [values, setValues] = useState<SelectorValues>(initial);
  const [shown, setShown] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [failure, setFailure] = useState<string | undefined>();
  const [deleting, setDeleting] = useState(false);
  const privileges = useApi<Privilege[]>(!creating && canPrivileges ? PRIVILEGES : null);
  const users = (privileges.data ?? []).filter((privilege) => privilege.contentSelector === selector?.name);
  const preview = usePreview(values.expression, '*');
  const set = (patch: Partial<SelectorValues>) => setValues((current) => ({ ...current, ...patch }));
  const errors = validateSelector(t, values, creating);
  const dirty = JSON.stringify(values) !== JSON.stringify(initial);
  const valid = Object.keys(errors).length === 0 && preview.problem === null;

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (Object.keys(errors).length > 0) {
      setShown(true);
      return;
    }
    setSubmitting(true);
    setFailure(undefined);
    try {
      if (creating) {
        const body = toCreateRequest(values);
        await api.post(SELECTORS, body);
        toast.success(t('selectors.form.created', { name: body.name }));
        await invalidate(SELECTORS);
        leave(LIST_PAGE);
        return;
      }
      await api.put(`${SELECTORS}${path`/${selector.name}`}`, toUpdateRequest(values));
      toast.success(t('selectors.form.saved', { name: selector.name }));
      await invalidate(SELECTORS);
    } catch (error) {
      setFailure(error instanceof Error ? error.message : String(error));
    } finally {
      setSubmitting(false);
    }
  };

  const remove = async () => {
    if (!selector) {
      return;
    }
    await api.delete(`${SELECTORS}${path`/${selector.name}`}`);
    toast.success(t('selectors.list.deleted', { name: selector.name }));
    leave(LIST_PAGE);
    await Promise.all([invalidate(SELECTORS), invalidate(PRIVILEGES)]);
  };

  const expressionText = values.expression.trim();
  return (
    <Page
      gutter="small"
      title={
        creating ? t('selectors.form.createTitle') : t('selectors.form.editTitle', { name: selector.name })
      }
      description={creating ? t('selectors.form.createIntro') : undefined}
      breadcrumbs={[
        { label: t('selectors.crumbs.settings'), to: '/settings' },
        { label: t('selectors.list.title'), to: LIST_PAGE },
        { label: creating ? t('selectors.form.create') : selector.name },
      ]}
    >
      <div className={creating ? styles.formWrap : `${styles.formWrap} ${styles.untitled}`}>
        <EditForm
          id="selector-form"
          tight
          onSubmit={submit}
          bar={
            <EditBar
              start={
                !creating &&
                canDelete && (
                  <Button
                    type="button"
                    size="3"
                    color="red"
                    variant="soft"
                    className={editStyles.bigButton}
                    disabled={users.length > 0}
                    onClick={() => setDeleting(true)}
                  >
                    <Trash2 size={16} aria-hidden="true" />
                    {t('selectors.form.delete')}
                  </Button>
                )
              }
              end={
                <>
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => (dirty ? setValues(initial) : navigate(LIST_PAGE))}
                  >
                    {t('selectors.form.discard')}
                  </Button>
                  {!readOnly && (
                    <Button type="submit" disabled={!dirty || !valid} loading={submitting}>
                      {creating ? t('selectors.form.createSubmit') : t('selectors.form.save')}
                    </Button>
                  )}
                </>
              }
            />
          }
        >
          {users.length > 0 && (
            <p className={styles.usage} role="note">
              <Info size={16} aria-hidden="true" />
              <Link2 size={16} aria-hidden="true" />
              <span>
                {t('selectors.form.usedBy', { count: users.length })}{' '}
                <strong>{users.map((privilege) => privilege.name).join(', ')}</strong>
              </span>
            </p>
          )}
          <FormSection title={t('selectors.form.settings')}>
            {creating ? (
              <Input
                label={t('selectors.form.name')}
                name="name"
                required
                help={t('selectors.form.nameHint')}
                value={values.name}
                autoFocus
                error={shown ? errors.name : undefined}
                onChange={(name) => set({ name })}
              />
            ) : (
              <ReadOnlyValue label={t('selectors.form.name')} value={selector.name} />
            )}
            <ReadOnlyValue label={t('selectors.list.type')} value="CSEL" />
            <Input
              label={t('selectors.form.description')}
              name="description"
              value={values.description}
              disabled={readOnly}
              onChange={(description) => set({ description })}
            />
          </FormSection>
          <FormSection title={t('selectors.form.expressionTitle')}>
            <p className={styles.intro}>{t('selectors.form.expressionIntro')}</p>
            <TextArea
              size="2"
              className={styles.expression}
              aria-label={t('selectors.form.expressionLabel')}
              placeholder={t('selectors.form.expressionPlaceholder')}
              value={values.expression}
              disabled={readOnly}
              onChange={(event) => set({ expression: event.target.value })}
            />
            {expressionText !== '' && preview.problem === null && preview.checked && (
              <p className={styles.valid}>
                <Check size={12} aria-hidden="true" />
                {t('selectors.form.valid')}
              </p>
            )}
            {preview.problem !== null && (
              <p className={styles.invalid} role="alert">
                <TriangleAlert size={12} aria-hidden="true" />
                {t('selectors.form.invalid', { message: preview.problem })}
              </p>
            )}
            {shown && errors.expression && (
              <p className={styles.invalid} role="alert">
                {errors.expression}
              </p>
            )}
            {expressionText !== '' && preview.problem === null && (
              <div className={styles.explain}>
                <span className={styles.explainTitle}>
                  <Eye size={12} aria-hidden="true" />
                  {t('selectors.form.explainTitle')}
                </span>
                <p>{explainExpression(t, expressionText)}</p>
              </div>
            )}
          </FormSection>
          <section className={styles.examples}>
            <h4 className={styles.examplesTitle}>
              {t('selectors.csel.examples')}
              <span>{t('selectors.form.examplesHint')}</span>
            </h4>
            <div className={styles.chips}>
              {EXAMPLES.map((example) => (
                <button
                  key={example}
                  type="button"
                  className={styles.example}
                  disabled={readOnly}
                  onClick={() => set({ expression: example })}
                >
                  <Plus size={12} aria-hidden="true" />
                  <code>{example}</code>
                </button>
              ))}
            </div>
          </section>
          <Reference />
          {!creating && <Results expression={expressionText} />}
          <FormError message={failure} />
        </EditForm>
      </div>
      <UnsavedChangesGuard when={dirty && !leaving && !readOnly && !submitting} />
      <ConfirmDialog
        open={deleting}
        onOpenChange={setDeleting}
        onConfirm={remove}
        title={t('selectors.list.deleteTitle', { name: selector?.name ?? '' })}
        description={t('selectors.list.deleteText')}
        confirmLabel={t('selectors.form.delete')}
        danger
      />
    </Page>
  );
}

export default function SelectorPage() {
  const { name } = useParams();
  const selector = useApi<ContentSelector>(name === undefined ? null : `${SELECTORS}${path`/${name}`}`);
  if (name === undefined) {
    return <SelectorForm key="create" />;
  }
  if (selector.isPending) {
    return (
      <Flex justify="center" py="8">
        <Spinner size="3" />
      </Flex>
    );
  }
  if (selector.error || !selector.data) {
    return (
      <Page gutter="small" title={name}>
        <ErrorState error={selector.error} onRetry={() => void selector.refetch()} />
      </Page>
    );
  }
  return <SelectorForm key={name} selector={selector.data} />;
}
