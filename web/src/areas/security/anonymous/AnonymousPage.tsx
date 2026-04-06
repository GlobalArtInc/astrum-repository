import { Button, Flex, Select, Spinner } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { Check } from 'lucide-react';
import { useId } from 'react';
import { api, useApi, useInvalidateApi } from '../../../api';
import { STATE_KEY, useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ErrorState, FormError, Page, UnsavedChangesGuard, useForm, useToast } from '../../../ui';
import { Input } from '../../identity/shared/Fields';
import fields from '../../identity/shared/Fields.module.css';
import { ANONYMOUS, type Anonymous } from '../api';
import { HelpCard } from '../shared/AdminList';
import { EditBar, EditForm, FormSection } from '../shared/Edit';
import edit from '../shared/Edit.module.css';
import { explainRefusal } from '../shared/messages';
import styles from './Anonymous.module.css';

export const AUTHORIZING_REALM = 'AstrumAuthorizingRealm';

const HELP = 'https://help.globalart.dev/en/anonymous-access.html';

const REALMS: Array<{ value: string; label: string }> = [
  { value: 'LdapRealm', label: 'LDAP Realm' },
  { value: AUTHORIZING_REALM, label: 'Local Authorizing Realm' },
  { value: 'OAuth2Realm', label: 'OpenID Connect Realm' },
];

export function anonymousRealms(current: string): Array<{ value: string; label: string }> {
  return current && !REALMS.some((realm) => realm.value === current)
    ? [...REALMS, { value: current, label: current }]
    : REALMS;
}

function AnonymousForm({ settings }: { settings: Anonymous }) {
  const t = useT();
  const toast = useToast();
  const client = useQueryClient();
  const invalidate = useInvalidateApi();
  const canWrite = useCan('astrum:settings:update');
  const readOnly = !canWrite;
  const enabledId = useId();
  const realmId = useId();
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
  const enabled = form.values.enabled;
  return (
    <EditForm
      id="anonymous-form"
      onSubmit={(event) => void form.submit(event)}
      bar={
        <EditBar
          end={
            !readOnly && (
              <>
                <Button type="button" variant="outline" onClick={() => form.reset()}>
                  {t('security.form.discard')}
                </Button>
                <Button type="submit" disabled={!form.dirty} loading={form.submitting}>
                  {t('security.form.save')}
                </Button>
              </>
            )
          }
        />
      }
    >
      <FormSection title={t('security.anonymous.section')}>
        <p className={styles.intro}>{t('security.anonymous.sectionText')}</p>
        <label className={styles.check} htmlFor={enabledId} data-disabled={readOnly || undefined}>
          <span className={enabled ? `${edit.mark} ${edit.markOn}` : edit.mark}>
            <input
              id={enabledId}
              type="checkbox"
              checked={enabled}
              disabled={readOnly}
              aria-labelledby={`${enabledId}-label`}
              aria-describedby={`${enabledId}-help`}
              onChange={(event) => form.set('enabled', event.target.checked)}
            />
            {enabled && <Check size={14} aria-hidden="true" />}
          </span>
          <span className={styles.checkText}>
            <span id={`${enabledId}-label`} className={styles.checkLabel}>
              {t('security.anonymous.enabled')}
            </span>
            <span id={`${enabledId}-help`} className={styles.checkHelp}>
              {t('security.anonymous.enabledText')}
            </span>
          </span>
        </label>
        <Input
          label={t('security.anonymous.userId')}
          help={t('security.anonymous.userIdHint')}
          required
          placeholder="anonymous"
          disabled={readOnly}
          {...form.field('userId')}
        />
        <div className={fields.field}>
          <label htmlFor={realmId} className={fields.label}>
            {t('security.anonymous.realm')}
            <span className={fields.required} aria-hidden="true">
              *
            </span>
          </label>
          <p id={`${realmId}-help`} className={fields.help}>
            {t('security.anonymous.realmHint')}
          </p>
          <Select.Root
            value={form.values.realmName || undefined}
            onValueChange={(value) => form.set('realmName', value)}
            disabled={readOnly}
            name="realmName"
          >
            <Select.Trigger
              id={realmId}
              className={styles.select}
              aria-describedby={`${realmId}-help`}
              aria-invalid={form.errors.realmName ? true : undefined}
            />
            <Select.Content position="popper">
              {anonymousRealms(form.values.realmName).map((realm) => (
                <Select.Item key={realm.value} value={realm.value}>
                  {realm.label}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
          {form.errors.realmName && (
            <p className={fields.error} role="alert">
              {form.errors.realmName}
            </p>
          )}
        </div>
        <FormError message={form.formError} />
      </FormSection>
      <hr className={styles.rule} />
      <HelpCard flush title={t('security.anonymous.helpTitle')} link={HELP}>
        {t('security.anonymous.about')}
      </HelpCard>
      <UnsavedChangesGuard when={form.dirty && !readOnly} />
    </EditForm>
  );
}

export default function AnonymousPage() {
  const t = useT();
  const settings = useApi<Anonymous>(ANONYMOUS);
  return (
    <Page
      className={edit.settingsPage}
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
