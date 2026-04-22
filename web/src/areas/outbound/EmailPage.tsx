import { Button, Flex, TextField as RadixTextField, Spinner, Text } from '@radix-ui/themes';
import { Send, ShieldCheck } from 'lucide-react';
import { useMemo, useState } from 'react';
import { api, useApi, useInvalidateApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import {
  CheckboxField,
  describeError,
  ErrorState,
  FormError,
  NumberField,
  Page,
  SecretField,
  TextField,
  UnsavedChangesGuard,
  useForm,
  useToast,
} from '../../ui';
import { SettingsFields, SettingsForm, SettingsSection, settingsButton } from '../../ui/SettingsForm';
import {
  EMAIL_DOCS,
  EMAIL_SETTINGS,
  EMAIL_VERIFY,
  type EmailSettings,
  PLACEHOLDER,
  postText,
  SETTINGS_UPDATE,
  TRUSTSTORE_READ,
} from './api';
import { AboutCard, Collapsible, DocsLink, ResultBox } from './components';
import {
  ADDRESS,
  type EmailValues,
  emailField,
  fromEmail,
  renameFieldErrors,
  storedPassword,
  toEmail,
  validateEmail,
} from './model';
import styles from './Outbound.module.css';
import { CertificateDialog } from './ssl/CertificateDialog';

type Outcome = { tone: 'success' | 'error'; text: string } | null;

function TestEmail({ enabled, readOnly }: { enabled: boolean; readOnly: boolean }) {
  const t = useT();
  const [address, setAddress] = useState('');
  const [sending, setSending] = useState(false);
  const [outcome, setOutcome] = useState<Outcome>(null);
  const send = async () => {
    const target = address.trim();
    if (!ADDRESS.test(target)) {
      setOutcome({ tone: 'error', text: t('outbound.email.problems.address') });
      return;
    }
    setSending(true);
    setOutcome(null);
    try {
      const reply = await postText(EMAIL_VERIFY, target);
      const body = (reply.body ?? {}) as { success?: boolean; reason?: string | null; message?: string };
      if (reply.status >= 200 && reply.status < 300 && body.success !== false) {
        setOutcome({ tone: 'success', text: t('outbound.email.testSent') });
      } else {
        const reason =
          body.reason ?? body.message ?? (typeof reply.body === 'string' ? reply.body : '') ?? '';
        setOutcome({
          tone: 'error',
          text: reason ? t('outbound.email.testFailedReason', { reason }) : t('outbound.email.testFailed'),
        });
      }
    } catch (failure) {
      setOutcome({
        tone: 'error',
        text: t('outbound.email.testFailedReason', { reason: describeError(t, failure) }),
      });
    } finally {
      setSending(false);
    }
  };
  const disabled = !enabled || readOnly;
  return (
    <SettingsSection title={t('outbound.email.test')}>
      <p className={styles.testText}>{t('outbound.email.testText')}</p>
      <div className={styles.testRow}>
        <div className={styles.testInput}>
          <label htmlFor="outbound-test-email" className={styles.inlineLabel}>
            {t('outbound.email.testAddress')}
          </label>
          <Text as="p" size="1" color="gray" id="outbound-test-email-hint" className={styles.inlineHint}>
            {t('outbound.email.testAddressHint')}
          </Text>
          <RadixTextField.Root
            id="outbound-test-email"
            type="email"
            value={address}
            placeholder="test@example.com"
            disabled={disabled}
            aria-describedby="outbound-test-email-hint"
            onChange={(event) => {
              setAddress(event.target.value);
              setOutcome(null);
            }}
            onKeyDown={(event) => {
              if (event.key === 'Enter') {
                event.preventDefault();
                if (address.trim() !== '' && !sending) {
                  void send();
                }
              }
            }}
          />
        </div>
        <Button
          className={`${settingsButton} ${styles.iconGap} ${styles.secondary}`}
          type="button"
          variant="outline"
          color="gray"
          size="3"
          loading={sending}
          disabled={disabled || address.trim() === ''}
          onClick={() => void send()}
        >
          <Send size={16} aria-hidden="true" />
          {t('outbound.email.send')}
        </Button>
      </div>
      {outcome && (
        <div className={styles.testResult}>
          <ResultBox tone={outcome.tone}>{outcome.text}</ResultBox>
        </div>
      )}
    </SettingsSection>
  );
}

function EmailForm({
  settings,
  onSaved,
}: {
  settings: EmailSettings;
  onSaved(settings: EmailSettings): void;
}) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const readOnly = !useCan(SETTINGS_UPDATE);
  const canViewCertificate = useCan(TRUSTSTORE_READ);
  const stored = storedPassword(settings.password);
  const initial = useMemo(() => fromEmail(settings), [settings]);
  const [tlsOpen, setTlsOpen] = useState(
    settings.startTlsEnabled ||
      settings.startTlsRequired ||
      settings.sslOnConnectEnabled ||
      settings.sslServerIdentityCheckEnabled,
  );
  const [viewing, setViewing] = useState(false);
  const texts = {
    host: t('outbound.email.problems.host'),
    port: t('outbound.email.problems.port'),
    portRange: t('outbound.email.problems.portRange'),
    from: t('outbound.email.problems.from'),
    address: t('outbound.email.problems.address'),
  };
  const form = useForm<EmailValues>({
    initial,
    validate: (values) => validateEmail(values, texts),
    onSubmit: async (values) => {
      const body = toEmail(values, stored);
      try {
        await api.put(EMAIL_SETTINGS, body);
      } catch (error) {
        throw renameFieldErrors(error, emailField);
      }
      toast.success(t('outbound.email.saved'));
      await invalidate(EMAIL_SETTINGS);
      onSaved({
        ...body,
        password: body.password === null ? null : PLACEHOLDER,
      });
    },
  });
  const field = <K extends keyof EmailValues & string>(name: K) => ({
    ...form.field(name),
    disabled: readOnly,
  });
  const values = form.values;
  return (
    <SettingsForm
      wide
      label={t('outbound.email.title')}
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
      <SettingsSection title={t('outbound.email.smtp')}>
        <SettingsFields>
          <CheckboxField
            {...field('enabled')}
            label={t('outbound.email.enable')}
            description={t('outbound.email.enableText')}
          />
          <TextField
            {...field('host')}
            label={t('outbound.email.host')}
            hint={t('outbound.email.hostHint')}
            placeholder="smtp.example.com"
            required
          />
          <NumberField
            {...field('port')}
            label={t('outbound.email.port')}
            hint={t('outbound.email.portHint')}
            min={1}
            max={65535}
            required
          />
          <div className={styles.trustRow}>
            <CheckboxField
              {...field('astrumTrustStoreEnabled')}
              label={t('outbound.email.trustStore')}
              description={t('outbound.email.trustStoreText')}
            />
            {canViewCertificate && (
              <Button
                className={`${settingsButton} ${styles.iconGap} ${styles.secondary}`}
                type="button"
                variant="outline"
                color="gray"
                size="3"
                disabled={values.host.trim() === '' || values.port === null}
                onClick={() => setViewing(true)}
              >
                <ShieldCheck size={16} aria-hidden="true" />
                {t('outbound.common.viewCertificate')}
              </Button>
            )}
          </div>
        </SettingsFields>
      </SettingsSection>
      <SettingsSection title={t('outbound.email.authentication')}>
        <SettingsFields>
          <CheckboxField
            {...field('useAuthentication')}
            label={t('outbound.email.enableAuthentication')}
            description={t('outbound.email.enableAuthenticationText')}
          />
          {values.useAuthentication && (
            <>
              <TextField
                {...field('username')}
                label={t('outbound.email.username')}
                hint={t('outbound.email.usernameHint')}
                placeholder="astrum-smtp-user"
                autoComplete="off"
              />
              <SecretField
                {...field('password')}
                label={t('outbound.email.password')}
                hint={stored ? t('outbound.email.passwordStored') : t('outbound.email.passwordHint')}
                stored={stored}
              />
            </>
          )}
        </SettingsFields>
      </SettingsSection>
      <SettingsSection title={t('outbound.email.emailSettings')}>
        <SettingsFields>
          <TextField
            {...field('fromAddress')}
            type="email"
            label={t('outbound.email.from')}
            hint={t('outbound.email.fromHint')}
            placeholder="astrum@example.com"
            required
          />
          <TextField
            {...field('subjectPrefix')}
            label={t('outbound.email.subjectPrefix')}
            hint={t('outbound.email.subjectPrefixHint')}
            placeholder="[Astrum]"
          />
        </SettingsFields>
      </SettingsSection>
      <div className={styles.tlsSection}>
        <Collapsible title={t('outbound.email.tls')} open={tlsOpen} onToggle={() => setTlsOpen(!tlsOpen)}>
          <SettingsFields>
            <CheckboxField
              {...field('startTlsEnabled')}
              label={t('outbound.email.startTls')}
              description={t('outbound.email.startTlsText')}
            />
            <CheckboxField
              {...field('startTlsRequired')}
              label={t('outbound.email.startTlsRequired')}
              description={t('outbound.email.startTlsRequiredText')}
            />
            <CheckboxField
              {...field('sslOnConnectEnabled')}
              label={t('outbound.email.sslOnConnect')}
              description={t('outbound.email.sslOnConnectText')}
            />
            <CheckboxField
              {...field('sslServerIdentityCheckEnabled')}
              label={t('outbound.email.identityCheck')}
              description={t('outbound.email.identityCheckText')}
            />
          </SettingsFields>
        </Collapsible>
      </div>
      <FormError message={form.formError} />
      <div className={styles.testSection}>
        <TestEmail enabled={values.enabled} readOnly={readOnly} />
      </div>
      <div className={styles.inlineAbout}>
        <AboutCard title={t('outbound.email.about')}>
          <p className={`${styles.aboutText} ${styles.aboutTextLoose}`}>
            {t('outbound.email.aboutText')}{' '}
            {t.rich('outbound.common.seeDocs', {
              link: (
                <DocsLink key="docs" href={EMAIL_DOCS}>
                  {t('outbound.common.docs')}
                </DocsLink>
              ),
            })}
          </p>
        </AboutCard>
      </div>
      {viewing && values.port !== null && (
        <CertificateDialog
          open={viewing}
          onOpenChange={setViewing}
          host={values.host.trim()}
          port={values.port}
        />
      )}
    </SettingsForm>
  );
}

export default function EmailPage() {
  const t = useT();
  const settings = useApi<EmailSettings>(EMAIL_SETTINGS);
  const [saved, setSaved] = useState<{ version: number; settings: EmailSettings } | null>(null);
  return (
    <Page
      className={`${styles.flushPage} ${styles.settingsPage}`}
      gutter="small"
      title={t('outbound.email.title')}
      description={t('outbound.email.description')}
      breadcrumbs={[
        { label: t('outbound.crumbs.settings'), to: '/settings' },
        { label: t('outbound.email.title') },
      ]}
    >
      {settings.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : settings.error ? (
        <ErrorState error={settings.error} onRetry={() => void settings.refetch()} />
      ) : (
        <EmailForm
          key={saved?.version ?? 0}
          settings={saved?.settings ?? settings.data}
          onSaved={(next) => setSaved({ version: (saved?.version ?? 0) + 1, settings: next })}
        />
      )}
    </Page>
  );
}
