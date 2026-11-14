import { Button, Callout, Card, Flex, Spinner, Text } from '@radix-ui/themes';
import { TriangleAlert } from 'lucide-react';
import { api, useApi, useInvalidateApi } from '../../../api';
import { type Translate, useT } from '../../../i18n';
import { ErrorState, FormError, Page, UnsavedChangesGuard, useForm, useToast } from '../../../ui';
import { IP_ALLOW, type IpAllowLists, type IpAllowView } from '../api';
import formStyles from '../shared/Form.module.css';
import { explainRefusal } from '../shared/messages';
import { ListEditor } from '../ssrf/ListEditor';
import styles from './IpAllow.module.css';
import { entryProblem, localizeLockout } from './model';

function lockoutMessage(t: Translate, address: string | null): string {
  return address === null ? t('security.ipAllow.lockoutUnknown') : t('security.ipAllow.lockout', { address });
}

function YourAddress({ view }: { view: IpAllowView }) {
  const t = useT();
  const unknown = t('security.ipAllow.unknown');
  return (
    <Card size="3">
      <div className={formStyles.card}>
        <Text as="p" size="3" weight="medium">
          {t('security.ipAllow.yourAddress')}
        </Text>
        <Text as="p" size="2" color="gray">
          {t('security.ipAllow.yourAddressText')}
        </Text>
        <dl className={styles.facts}>
          <dt>{t('security.ipAllow.address')}</dt>
          <dd className={formStyles.mono}>{view.yourAddress ?? unknown}</dd>
          <dt>{t('security.ipAllow.peer')}</dt>
          <dd className={formStyles.mono}>{view.peer ?? unknown}</dd>
          <dt>{t('security.ipAllow.trustedProxies')}</dt>
          <dd className={formStyles.mono}>
            {view.trustedProxies.length > 0
              ? view.trustedProxies.join(', ')
              : t('security.ipAllow.noProxies')}
          </dd>
        </dl>
        {view.forwardedIgnored && (
          <Callout.Root color="amber" role="status">
            <Callout.Icon>
              <TriangleAlert size={16} />
            </Callout.Icon>
            <Callout.Text>
              {t('security.ipAllow.forwardedIgnored', { peer: view.peer ?? unknown })}
            </Callout.Text>
          </Callout.Root>
        )}
      </div>
    </Card>
  );
}

function IpAllowForm({ view }: { view: IpAllowView }) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const form = useForm<IpAllowLists>({
    initial: { management: view.management, content: view.content, agents: view.agents },
    onSubmit: (values) =>
      explainRefusal(t, async () => {
        try {
          await api.put<IpAllowView>(IP_ALLOW, values);
        } catch (failure) {
          throw localizeLockout(failure, lockoutMessage(t, view.yourAddress));
        }
        toast.success(t('security.ipAllow.saved'));
        await invalidate(IP_ALLOW);
      }),
  });
  const check = (value: string) => {
    const problem = entryProblem(value);
    if (problem === undefined) {
      return undefined;
    }
    return problem === 'wholeFamily'
      ? t('security.ipAllow.wholeFamily', { value })
      : t('security.ipAllow.badEntry', { value });
  };
  return (
    <form className={formStyles.form} onSubmit={form.submit} noValidate>
      <Card size="3">
        <div className={formStyles.card}>
          <ListEditor
            label={t('security.ipAllow.management')}
            hint={t('security.ipAllow.managementHint')}
            placeholder="10.0.0.0/8"
            values={form.values.management}
            onChange={(values) => form.set('management', values)}
            check={check}
            error={form.errors.management}
            emptyText={t('security.ipAllow.open')}
          />
          <ListEditor
            label={t('security.ipAllow.content')}
            hint={t('security.ipAllow.contentHint')}
            placeholder="10.0.0.0/8"
            values={form.values.content}
            onChange={(values) => form.set('content', values)}
            check={check}
            error={form.errors.content}
            emptyText={t('security.ipAllow.open')}
          />
          <ListEditor
            label={t('security.ipAllow.agents')}
            hint={t('security.ipAllow.agentsHint')}
            placeholder="10.0.0.0/8"
            values={form.values.agents}
            onChange={(values) => form.set('agents', values)}
            check={check}
            error={form.errors.agents}
            emptyText={t('security.ipAllow.open')}
          />
        </div>
      </Card>
      <FormError message={form.formError} />
      <div className={formStyles.actions}>
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

export default function IpAllowPage() {
  const t = useT();
  const view = useApi<IpAllowView>(IP_ALLOW);
  return (
    <Page
      title={t('security.ipAllow.title')}
      description={t('security.ipAllow.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.ipAllow.title') },
      ]}
    >
      {view.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : view.error ? (
        <ErrorState error={view.error} onRetry={() => void view.refetch()} />
      ) : (
        <Flex direction="column" gap="4">
          <YourAddress view={view.data} />
          <IpAllowForm view={view.data} />
        </Flex>
      )}
    </Page>
  );
}
