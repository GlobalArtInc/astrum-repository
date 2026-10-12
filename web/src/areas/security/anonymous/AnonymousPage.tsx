import { Button, Callout, Card, Flex, Spinner } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { Info } from 'lucide-react';
import { api, useApi, useInvalidateApi } from '../../../api';
import { STATE_KEY, useCan } from '../../../auth/state';
import { type Translate, useT } from '../../../i18n';
import {
  ErrorState,
  FormError,
  type Option,
  Page,
  SelectField,
  SwitchField,
  TextField,
  UnsavedChangesGuard,
  useForm,
  useToast,
} from '../../../ui';
import { ANONYMOUS, type Anonymous } from '../api';
import styles from '../shared/Form.module.css';
import { explainRefusal } from '../shared/messages';

export const AUTHORIZING_REALM = 'AstrumAuthorizingRealm';

const REALM_LABELS: Record<string, string> = {
  AstrumAuthorizingRealm: 'security.anonymous.realmLocal',
  LdapRealm: 'security.anonymous.realmLdap',
  OAuth2Realm: 'security.anonymous.realmOidc',
};

export function anonymousRealms(t: Translate, current: string): Option[] {
  const ids = Object.keys(REALM_LABELS);
  if (current && !ids.includes(current)) {
    ids.push(current);
  }
  return ids.map((id) => {
    const key = REALM_LABELS[id];
    return { value: id, label: key ? t(key) : id };
  });
}

function AnonymousForm({ settings }: { settings: Anonymous }) {
  const t = useT();
  const toast = useToast();
  const client = useQueryClient();
  const invalidate = useInvalidateApi();
  const canWrite = useCan('astrum:settings:update');
  const form = useForm<Anonymous>({
    initial: settings,
    validate: (values) => {
      const errors: Partial<Record<keyof Anonymous, string>> = {};
      if (values.userId.trim() === '') {
        errors.userId = t('security.form.required');
      }
      if (values.realmName.trim() === '') {
        errors.realmName = t('security.form.required');
      }
      return errors;
    },
    onSubmit: (values) =>
      explainRefusal(t, async () => {
        await api.put(ANONYMOUS, { ...values, userId: values.userId.trim() });
        toast.success(values.enabled ? t('security.anonymous.savedOn') : t('security.anonymous.savedOff'));
        await Promise.all([invalidate(ANONYMOUS), client.invalidateQueries({ queryKey: STATE_KEY })]);
      }),
  });
  return (
    <form className={styles.form} onSubmit={form.submit} noValidate>
      <Card size="3">
        <div className={styles.card}>
          <SwitchField
            label={t('security.anonymous.enabled')}
            description={t('security.anonymous.enabledText')}
            {...form.field('enabled')}
            disabled={!canWrite}
          />
          <TextField
            label={t('security.anonymous.userId')}
            hint={t('security.anonymous.userIdHint')}
            {...form.field('userId')}
            required
            disabled={!canWrite}
            autoComplete="off"
          />
          <SelectField
            label={t('security.anonymous.realm')}
            hint={t('security.anonymous.realmHint')}
            {...form.field('realmName')}
            options={anonymousRealms(t, form.values.realmName)}
            required
            disabled={!canWrite}
          />
        </div>
      </Card>
      <Callout.Root color="gray" variant="surface">
        <Callout.Icon>
          <Info size={16} />
        </Callout.Icon>
        <Callout.Text>{t('security.anonymous.about')}</Callout.Text>
      </Callout.Root>
      <FormError message={form.formError} />
      {canWrite && (
        <div className={styles.actions}>
          <Button type="submit" loading={form.submitting} disabled={!form.dirty}>
            {t('security.form.save')}
          </Button>
          <Button
            type="button"
            variant="soft"
            color="gray"
            disabled={!form.dirty}
            onClick={() => form.reset()}
          >
            {t('security.form.discard')}
          </Button>
        </div>
      )}
      <UnsavedChangesGuard when={form.dirty && canWrite} />
    </form>
  );
}

export default function AnonymousPage() {
  const t = useT();
  const settings = useApi<Anonymous>(ANONYMOUS);
  return (
    <Page
      title={t('security.anonymous.title')}
      description={t('security.anonymous.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.anonymous.title') },
      ]}
    >
      {settings.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : settings.error ? (
        <ErrorState error={settings.error} onRetry={() => void settings.refetch()} />
      ) : (
        <AnonymousForm settings={settings.data} />
      )}
    </Page>
  );
}
