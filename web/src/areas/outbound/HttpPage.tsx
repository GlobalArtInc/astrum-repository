import { Button, Flex, IconButton, TextField as RadixTextField, Spinner, Text } from '@radix-ui/themes';
import { Plus, Trash2 } from 'lucide-react';
import { useMemo, useState } from 'react';
import { api, useApi, useInvalidateApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import {
  CheckboxField,
  ErrorState,
  FormError,
  NumberField,
  Page,
  SecretField,
  SelectField,
  TextField,
  UnsavedChangesGuard,
  useForm,
  useToast,
} from '../../ui';
import { SettingsFields, SettingsForm, SettingsSection, settingsButton } from '../../ui/SettingsForm';
import { HTTP_DOCS, HTTP_SETTINGS, type HttpSettings, SETTINGS_UPDATE } from './api';
import { AboutCard, Collapsible, DocsLink } from './components';
import {
  type AuthType,
  fromHttp,
  type HttpValues,
  hostPattern,
  httpField,
  type ProxyKind,
  renameFieldErrors,
  storedPassword,
  toHttp,
  validateHttp,
} from './model';
import styles from './Outbound.module.css';

function ProxySection({
  kind,
  form,
  stored,
  readOnly,
}: {
  kind: ProxyKind;
  form: ReturnType<typeof useForm<HttpValues>>;
  stored: boolean;
  readOnly: boolean;
}) {
  const t = useT();
  const values = form.values;
  const [open, setOpen] = useState(values[`${kind}Auth`] === 'username');
  const enabled = values[`${kind}Enabled`];
  const field = <K extends keyof HttpValues & string>(name: K) => ({
    ...form.field(name),
    disabled: readOnly,
  });
  const texts =
    kind === 'http'
      ? {
          title: t('outbound.http.httpProxy'),
          enable: t('outbound.http.enableHttp'),
          enableText: t('outbound.http.enableHttpText'),
          host: t('outbound.http.httpHost'),
          hostHint: t('outbound.http.httpHostHint'),
          port: t('outbound.http.httpPort'),
          auth: t('outbound.http.httpAuthentication'),
          username: t('outbound.http.httpUsernameHint'),
          password: t('outbound.http.httpPasswordHint'),
          portPlaceholder: '8080',
        }
      : {
          title: t('outbound.http.httpsProxy'),
          enable: t('outbound.http.enableHttps'),
          enableText: t('outbound.http.enableHttpsText'),
          host: t('outbound.http.httpsHost'),
          hostHint: t('outbound.http.httpsHostHint'),
          port: t('outbound.http.httpsPort'),
          auth: t('outbound.http.httpsAuthentication'),
          username: t('outbound.http.httpsUsernameHint'),
          password: t('outbound.http.httpsPasswordHint'),
          portPlaceholder: '8443',
        };
  return (
    <SettingsSection title={texts.title}>
      <SettingsFields>
        <CheckboxField {...field(`${kind}Enabled`)} label={texts.enable} description={texts.enableText} />
        {enabled && (
          <>
            <TextField
              {...field(`${kind}Host`)}
              label={texts.host}
              hint={texts.hostHint}
              placeholder="proxy.example.com"
              required
            />
            <TextField
              {...field(`${kind}Port`)}
              label={texts.port}
              placeholder={texts.portPlaceholder}
              required
            />
            <Collapsible title={texts.auth} open={open} onToggle={() => setOpen(!open)}>
              <SettingsFields>
                <SelectField
                  {...field(`${kind}Auth`)}
                  onChange={(value) => form.set(`${kind}Auth`, value as AuthType)}
                  label={t('outbound.http.authentication')}
                  hint={t('outbound.http.authenticationHint')}
                  options={[
                    { value: 'none', label: t('outbound.http.noAuthentication') },
                    { value: 'username', label: t('outbound.http.username') },
                  ]}
                />
                {values[`${kind}Auth`] === 'username' && (
                  <>
                    <TextField
                      {...field(`${kind}Username`)}
                      label={t('outbound.http.username')}
                      hint={texts.username}
                      placeholder="proxy-user"
                      autoComplete="off"
                      required
                    />
                    <SecretField
                      {...field(`${kind}Password`)}
                      label={t('outbound.http.password')}
                      hint={texts.password}
                      stored={stored}
                    />
                    <TextField
                      {...field(`${kind}NtlmHost`)}
                      label={t('outbound.http.ntlmHost')}
                      hint={t('outbound.http.ntlmHostHint')}
                      placeholder="WORKSTATION01"
                    />
                    <TextField
                      {...field(`${kind}NtlmDomain`)}
                      label={t('outbound.http.ntlmDomain')}
                      hint={t('outbound.http.ntlmDomainHint')}
                      placeholder="COMPANY"
                    />
                  </>
                )}
              </SettingsFields>
            </Collapsible>
          </>
        )}
      </SettingsFields>
    </SettingsSection>
  );
}

function NonProxyHosts({
  hosts,
  onChange,
  readOnly,
  error,
}: {
  hosts: string[];
  onChange(hosts: string[]): void;
  readOnly: boolean;
  error?: string;
}) {
  const t = useT();
  const [draft, setDraft] = useState('');
  const pattern = hostPattern(draft);
  const add = () => {
    if (pattern && !hosts.includes(pattern)) {
      onChange([...hosts, pattern]);
    }
    setDraft('');
  };
  return (
    <SettingsSection
      title={
        hosts.length > 0
          ? t('outbound.http.excludeCount', { count: hosts.length })
          : t('outbound.http.exclude')
      }
    >
      <p className={styles.excludeText}>{t('outbound.http.excludeText')}</p>
      {!readOnly && (
        <div className={styles.addRow}>
          <div className={styles.addInput}>
            <label htmlFor="outbound-non-proxy-host" className={styles.inlineLabel}>
              {t('outbound.http.hostPattern')}
            </label>
            <Text
              as="p"
              size="1"
              color="gray"
              id="outbound-non-proxy-host-hint"
              className={styles.inlineHint}
            >
              {t('outbound.http.hostPatternHint')}
            </Text>
            <RadixTextField.Root
              id="outbound-non-proxy-host"
              value={draft}
              placeholder="*.example.com"
              aria-describedby="outbound-non-proxy-host-hint"
              onChange={(event) => setDraft(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') {
                  event.preventDefault();
                  add();
                }
              }}
            />
          </div>
          <Button
            className={settingsButton}
            type="button"
            variant="outline"
            color="gray"
            size="3"
            disabled={!pattern}
            onClick={add}
          >
            <Plus size={16} aria-hidden="true" />
            {t('outbound.http.add')}
          </Button>
        </div>
      )}
      {hosts.length > 0 && (
        <ul className={styles.hosts} aria-label={t('outbound.http.exclude')}>
          {hosts.map((host) => (
            <li key={host} className={styles.host}>
              <span>{host}</span>
              {!readOnly && (
                <IconButton
                  type="button"
                  variant="ghost"
                  color="gray"
                  aria-label={t('outbound.http.remove', { host })}
                  onClick={() => onChange(hosts.filter((other) => other !== host))}
                >
                  <Trash2 size={16} aria-hidden="true" />
                </IconButton>
              )}
            </li>
          ))}
        </ul>
      )}
      {error && (
        <Text as="p" size="1" color="red" role="alert" mt="2">
          {error}
        </Text>
      )}
    </SettingsSection>
  );
}

function HttpForm({ settings, onSaved }: { settings: HttpSettings; onSaved(settings: HttpSettings): void }) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const readOnly = !useCan(SETTINGS_UPDATE);
  const stored = {
    http: storedPassword(settings.httpProxy?.authInfo?.password),
    https: storedPassword(settings.httpsProxy?.authInfo?.password),
  };
  const initial = useMemo(() => fromHttp(settings), [settings]);
  const texts = {
    timeout: t('outbound.http.problems.timeout'),
    retries: t('outbound.http.problems.retries'),
    hostRequired: {
      http: t('outbound.http.problems.httpHost'),
      https: t('outbound.http.problems.httpsHost'),
    },
    portRequired: {
      http: t('outbound.http.problems.httpPort'),
      https: t('outbound.http.problems.httpsPort'),
    },
    hostShape: t('outbound.http.problems.hostShape'),
    port: t('outbound.http.problems.port'),
    username: t('outbound.http.problems.username'),
  };
  const form = useForm<HttpValues>({
    initial,
    validate: (values) => validateHttp(values, texts),
    onSubmit: async (values) => {
      let saved: HttpSettings;
      try {
        saved = await api.put<HttpSettings>(HTTP_SETTINGS, toHttp(values, stored));
      } catch (error) {
        throw renameFieldErrors(error, httpField);
      }
      toast.success(t('outbound.http.saved'));
      await invalidate(HTTP_SETTINGS);
      onSaved(saved ?? toHttp(values, stored));
    },
  });
  const anyProxy = form.values.httpEnabled || form.values.httpsEnabled;
  return (
    <SettingsForm
      wide
      label={t('outbound.http.title')}
      onSubmit={() => void form.submit()}
      unsaved={form.dirty && !readOnly ? t('outbound.common.unsaved') : null}
      end={
        !readOnly && (
          <>
            <Button
              type="button"
              variant="outline"
              disabled={!form.dirty || form.submitting}
              onClick={() => form.reset()}
            >
              {t('outbound.common.discard')}
            </Button>
            <Button type="submit" loading={form.submitting} disabled={!form.dirty}>
              {t('outbound.common.save')}
            </Button>
          </>
        )
      }
    >
      <UnsavedChangesGuard when={form.dirty && !readOnly} />
      {readOnly && (
        <Text as="p" size="2" color="gray" className={styles.readOnly}>
          {t('outbound.http.readOnly')}
        </Text>
      )}
      <SettingsSection title={t('outbound.http.connection')}>
        <SettingsFields>
          <TextField
            {...form.field('userAgent')}
            disabled={readOnly}
            label={t('outbound.http.userAgent')}
            hint={t('outbound.http.userAgentHint')}
            placeholder="MyCompany/1.0"
          />
          <NumberField
            {...form.field('timeout')}
            disabled={readOnly}
            label={t('outbound.http.timeout')}
            hint={t('outbound.http.timeoutHint')}
            placeholder={t('outbound.http.systemDefault')}
            min={1}
            max={3600}
          />
          <NumberField
            {...form.field('retries')}
            disabled={readOnly}
            label={t('outbound.http.retries')}
            hint={t('outbound.http.retriesHint')}
            placeholder={t('outbound.http.systemDefault')}
            min={0}
            max={10}
          />
        </SettingsFields>
      </SettingsSection>
      <ProxySection kind="http" form={form} stored={stored.http} readOnly={readOnly} />
      <ProxySection kind="https" form={form} stored={stored.https} readOnly={readOnly} />
      {anyProxy && (
        <NonProxyHosts
          hosts={form.values.nonProxyHosts}
          onChange={(hosts) => form.set('nonProxyHosts', hosts)}
          readOnly={readOnly}
          error={form.errors.nonProxyHosts}
        />
      )}
      <FormError message={form.formError} />
      <div className={styles.aboutSpacer}>
        <AboutCard title={t('outbound.http.about')}>
          <p className={styles.aboutText}>{t('outbound.http.aboutText')}</p>
          <p className={styles.aboutText}>
            {t.rich('outbound.common.seeDocs', {
              link: (
                <DocsLink key="docs" href={HTTP_DOCS}>
                  {t('outbound.common.docs')}
                </DocsLink>
              ),
            })}
          </p>
        </AboutCard>
      </div>
    </SettingsForm>
  );
}

export default function HttpPage() {
  const t = useT();
  const settings = useApi<HttpSettings>(HTTP_SETTINGS);
  const [saved, setSaved] = useState<{ version: number; settings: HttpSettings } | null>(null);
  return (
    <Page
      className={`${styles.flushPage} ${styles.settingsPage}`}
      gutter="small"
      title={t('outbound.http.title')}
      description={t('outbound.http.description')}
      breadcrumbs={[
        { label: t('outbound.crumbs.settings'), to: '/settings' },
        { label: t('outbound.http.crumb') },
      ]}
    >
      {settings.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : settings.error ? (
        <ErrorState error={settings.error} onRetry={() => void settings.refetch()} />
      ) : (
        <HttpForm
          key={saved?.version ?? 0}
          settings={saved?.settings ?? settings.data}
          onSaved={(next) => setSaved({ version: (saved?.version ?? 0) + 1, settings: next })}
        />
      )}
    </Page>
  );
}
