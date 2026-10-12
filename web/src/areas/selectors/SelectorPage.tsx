import { Button, Card, Flex, Spinner } from '@radix-ui/themes';
import { Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import {
  ConfirmDialog,
  ErrorState,
  FormError,
  Page,
  TextField,
  UnsavedChangesGuard,
  useForm,
  useToast,
} from '../../ui';
import { CselEditor } from './CselEditor';
import {
  ALL_REPOSITORIES,
  type ContentSelector,
  EMPTY_SELECTOR,
  fromSelector,
  LIST_PAGE,
  PRIVILEGES,
  SELECTORS,
  type SelectorValues,
  selectorHref,
  toCreateRequest,
  toUpdateRequest,
  validateSelector,
} from './model';
import { SelectorPreview, usePreview } from './SelectorPreview';
import styles from './Selectors.module.css';

function SelectorForm({
  selector,
  leaving,
  leave,
}: {
  selector?: ContentSelector;
  leaving: boolean;
  leave(to: string): void;
}) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const creating = selector === undefined;
  const canWrite = useCan(creating ? 'astrum:selectors:create' : 'astrum:selectors:update');
  const canDelete = useCan('astrum:selectors:delete');
  const readOnly = !canWrite;
  const [repository, setRepository] = useState(ALL_REPOSITORIES);
  const [deleting, setDeleting] = useState(false);

  const form = useForm<SelectorValues>({
    initial: selector ? fromSelector(selector) : EMPTY_SELECTOR,
    validate: (values) => validateSelector(t, values, creating),
    onSubmit: async (values) => {
      if (creating) {
        const body = toCreateRequest(values);
        await api.post(SELECTORS, body);
        toast.success(t('selectors.form.created', { name: body.name }));
        await invalidate(SELECTORS);
        leave(selectorHref(body.name));
        return;
      }
      await api.put(`${SELECTORS}${path`/${selector.name}`}`, toUpdateRequest(values));
      toast.success(t('selectors.form.saved', { name: selector.name }));
      await invalidate(SELECTORS);
    },
  });
  const preview = usePreview(form.values.expression, repository);

  const remove = async () => {
    if (!selector) {
      return;
    }
    await api.delete(`${SELECTORS}${path`/${selector.name}`}`);
    toast.success(t('selectors.list.deleted', { name: selector.name }));
    leave(LIST_PAGE);
    await Promise.all([invalidate(SELECTORS), invalidate(PRIVILEGES)]);
  };

  return (
    <>
      <form className={styles.form} onSubmit={form.submit} noValidate>
        <Card size="3">
          <div className={styles.card}>
            <TextField
              label={t('selectors.form.name')}
              {...form.field('name')}
              required
              disabled={readOnly || !creating}
              autoFocus={creating}
              hint={creating ? t('selectors.form.nameHint') : undefined}
              maxLength={200}
            />
            <TextField
              label={t('selectors.form.description')}
              {...form.field('description')}
              disabled={readOnly}
            />
            <CselEditor
              value={form.values.expression}
              onChange={(value) => form.set('expression', value)}
              error={form.errors.expression}
              problem={preview.problem}
              valid={preview.checked}
              disabled={readOnly}
            />
          </div>
        </Card>
        <FormError message={form.formError} />
        <div className={styles.actions}>
          {!readOnly && (
            <Button type="submit" loading={form.submitting} disabled={!creating && !form.dirty}>
              {creating ? t('selectors.form.createSubmit') : t('selectors.form.save')}
            </Button>
          )}
          <Button asChild variant="soft" color="gray">
            <Link to={LIST_PAGE}>{readOnly ? t('selectors.form.back') : t('selectors.form.cancel')}</Link>
          </Button>
          {!creating && canDelete && (
            <Button type="button" variant="soft" color="red" onClick={() => setDeleting(true)}>
              <Trash2 size={16} aria-hidden="true" />
              {t('selectors.form.delete')}
            </Button>
          )}
        </div>
        <UnsavedChangesGuard when={form.dirty && !leaving && !readOnly} />
        <ConfirmDialog
          open={deleting}
          onOpenChange={setDeleting}
          onConfirm={remove}
          title={t('selectors.list.deleteTitle', { name: selector?.name ?? '' })}
          description={t('selectors.list.deleteText')}
          confirmLabel={t('selectors.form.delete')}
          danger
        />
      </form>
      <SelectorPreview state={preview} repository={repository} onRepositoryChange={setRepository} />
    </>
  );
}

export default function SelectorPage() {
  const t = useT();
  const { name } = useParams();
  const navigate = useNavigate();
  const [target, setTarget] = useState<string | null>(null);
  useEffect(() => {
    if (target !== null) {
      navigate(target);
    }
  }, [target, navigate]);
  const creating = name === undefined;
  const selector = useApi<ContentSelector>(creating ? null : `${SELECTORS}${path`/${name}`}`);
  return (
    <Page
      title={creating ? t('selectors.form.createTitle') : (name ?? '')}
      description={creating ? t('selectors.form.createIntro') : (selector.data?.description ?? undefined)}
      breadcrumbs={[
        { label: t('selectors.crumbs.settings'), to: '/settings' },
        { label: t('selectors.list.title'), to: LIST_PAGE },
        { label: creating ? t('selectors.form.create') : (name ?? '') },
      ]}
    >
      {creating ? (
        <SelectorForm key="create" leaving={target !== null} leave={setTarget} />
      ) : selector.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : selector.error ? (
        <ErrorState error={selector.error} />
      ) : (
        <SelectorForm key={name} selector={selector.data} leaving={target !== null} leave={setTarget} />
      )}
    </Page>
  );
}
