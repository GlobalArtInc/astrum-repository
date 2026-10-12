import { Button, Callout, Card, Flex, Spinner } from '@radix-ui/themes';
import { TriangleAlert } from 'lucide-react';
import { api, useApi, useInvalidateApi } from '../../../api';
import { useT } from '../../../i18n';
import {
  ErrorState,
  FormError,
  Page,
  SwitchField,
  UnsavedChangesGuard,
  useForm,
  useToast,
} from '../../../ui';
import { SSRF, type SsrfProtection } from '../api';
import styles from '../shared/Form.module.css';
import { explainRefusal } from '../shared/messages';
import { ListEditor } from './ListEditor';
import { normalizeDomain, validDomain, validNetwork } from './model';

function SsrfForm({ settings }: { settings: SsrfProtection }) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const form = useForm<SsrfProtection>({
    initial: settings,
    onSubmit: (values) =>
      explainRefusal(t, async () => {
        await api.put<SsrfProtection>(SSRF, values);
        toast.success(t('security.ssrf.saved'));
        await invalidate(SSRF);
      }),
  });
  return (
    <form className={styles.form} onSubmit={form.submit} noValidate>
      <Card size="3">
        <div className={styles.card}>
          <SwitchField
            label={t('security.ssrf.enabled')}
            description={t('security.ssrf.enabledText')}
            {...form.field('enabled')}
          />
          {!form.values.enabled && (
            <Callout.Root color="amber" role="status">
              <Callout.Icon>
                <TriangleAlert size={16} />
              </Callout.Icon>
              <Callout.Text>{t('security.ssrf.offWarning')}</Callout.Text>
            </Callout.Root>
          )}
          <ListEditor
            label={t('security.ssrf.ips')}
            hint={t('security.ssrf.ipsHint')}
            placeholder="10.0.0.0/8"
            values={form.values.allowedIPs}
            onChange={(values) => form.set('allowedIPs', values)}
            check={(value) => (validNetwork(value) ? undefined : t('security.ssrf.badIp', { value }))}
            error={form.errors.allowedIPs}
            emptyText={t('security.ssrf.noIps')}
          />
          <ListEditor
            label={t('security.ssrf.domains')}
            hint={t('security.ssrf.domainsHint')}
            placeholder="*.corp.example"
            values={form.values.allowedDomains}
            onChange={(values) => form.set('allowedDomains', values)}
            normalize={normalizeDomain}
            check={(value) => (validDomain(value) ? undefined : t('security.ssrf.badDomain', { value }))}
            error={form.errors.allowedDomains}
            emptyText={t('security.ssrf.noDomains')}
          />
        </div>
      </Card>
      <FormError message={form.formError} />
      <div className={styles.actions}>
        <Button type="submit" loading={form.submitting} disabled={!form.dirty}>
          {t('security.form.save')}
        </Button>
        <Button type="button" variant="soft" color="gray" disabled={!form.dirty} onClick={() => form.reset()}>
          {t('security.form.discard')}
        </Button>
      </div>
      <UnsavedChangesGuard when={form.dirty} />
    </form>
  );
}

export default function SsrfPage() {
  const t = useT();
  const settings = useApi<SsrfProtection>(SSRF);
  return (
    <Page
      title={t('security.ssrf.title')}
      description={t('security.ssrf.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.ssrf.title') },
      ]}
    >
      {settings.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : settings.error ? (
        <ErrorState error={settings.error} onRetry={() => void settings.refetch()} />
      ) : (
        <SsrfForm settings={settings.data} />
      )}
    </Page>
  );
}
