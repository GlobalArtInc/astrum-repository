import { Button, RadioGroup, Text, TextArea } from '@radix-ui/themes';
import { type FormEvent, useState } from 'react';
import { Link, useNavigate } from 'react-router';
import { ApiError, api, useApi, useInvalidateApi, withQuery } from '../../../api';
import { useT } from '../../../i18n';
import { describeError, FormError, Page, TextField, UnsavedChangesGuard, useToast } from '../../../ui';
import { SettingsFields, SettingsForm, SettingsSection } from '../../../ui/SettingsForm';
import {
  CERTIFICATE_DETAILS,
  type Certificate,
  certificateHref,
  SSL,
  SSL_LIST,
  sendText,
  TRUSTSTORE,
} from '../api';
import { certificateName, looksLikePem, parseServer } from '../model';
import styles from '../Outbound.module.css';
import { CertificateDetails } from './CertificateDetails';

type Source = 'server' | 'pem';

export default function AddCertificatePage() {
  const t = useT();
  const navigate = useNavigate();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const trusted = useApi<Certificate[]>(TRUSTSTORE);
  const [source, setSource] = useState<Source>('server');
  const [server, setServer] = useState('');
  const [pem, setPem] = useState('');
  const [problem, setProblem] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [loaded, setLoaded] = useState<Certificate | null>(null);
  const [leaving, setLeaving] = useState(false);
  const present = loaded ? (trusted.data ?? []).some((known) => known.id === loaded.id) : false;
  const ready = source === 'server' ? server.trim() !== '' : pem.trim() !== '';

  const load = async () => {
    setError(null);
    setProblem(null);
    if (source === 'server') {
      const target = parseServer(server);
      if (!target) {
        setProblem(server.trim() === '' ? t('outbound.ssl.serverRequired') : t('outbound.ssl.serverInvalid'));
        return;
      }
      setBusy(true);
      try {
        setLoaded(await api.get<Certificate>(withQuery(SSL, { host: target.host, port: target.port })));
      } catch (failure) {
        setError(`${t('outbound.ssl.retrieveFailed')}: ${describeError(t, failure)}`);
      } finally {
        setBusy(false);
      }
      return;
    }
    if (!looksLikePem(pem)) {
      setProblem(pem.trim() === '' ? t('outbound.ssl.pemRequired') : t('outbound.ssl.pemInvalid'));
      return;
    }
    setBusy(true);
    try {
      setLoaded(await sendText<Certificate>(CERTIFICATE_DETAILS, pem.trim()));
    } catch (failure) {
      setError(`${t('outbound.ssl.parseFailed')}: ${describeError(t, failure)}`);
    } finally {
      setBusy(false);
    }
  };

  const add = async () => {
    if (!loaded) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const created = await sendText<Certificate>(TRUSTSTORE, loaded.pem);
      toast.success(t('outbound.ssl.added', { name: certificateName(loaded) }));
      await invalidate(TRUSTSTORE);
      setLeaving(true);
      navigate(certificateHref(created?.id ?? loaded.id));
    } catch (failure) {
      if (failure instanceof ApiError && failure.status === 409) {
        setError(t('outbound.ssl.exists'));
        await invalidate(TRUSTSTORE);
      } else {
        setError(`${t('outbound.ssl.addFailed')}: ${describeError(t, failure)}`);
      }
    } finally {
      setBusy(false);
    }
  };

  const submit = () => {
    if (loaded) {
      void add();
    } else {
      void load();
    }
  };

  const discard = () => {
    if (loaded) {
      setLoaded(null);
      setError(null);
    } else {
      setLeaving(true);
      navigate(SSL_LIST);
    }
  };

  return (
    <Page
      className={styles.page}
      title={t('outbound.ssl.addTitle')}
      description={t('outbound.ssl.addDescription')}
      breadcrumbs={[
        { label: t('outbound.crumbs.settings'), to: '/settings' },
        { label: t('outbound.ssl.title'), to: SSL_LIST },
        { label: t('outbound.ssl.addCrumb') },
      ]}
    >
      <SettingsForm
        wide
        label={t('outbound.ssl.addTitle')}
        onSubmit={submit}
        unsaved={loaded ? t('outbound.common.unsaved') : null}
        end={
          <>
            <Button type="button" variant="outline" onClick={discard} disabled={busy}>
              {t('outbound.common.discard')}
            </Button>
            {loaded ? (
              <Button type="submit" loading={busy} disabled={present}>
                {t('outbound.ssl.addSubmit')}
              </Button>
            ) : (
              <Button type="submit" loading={busy} disabled={!ready}>
                {t('outbound.ssl.load')}
              </Button>
            )}
          </>
        }
      >
        <UnsavedChangesGuard when={loaded !== null && !leaving} />
        {loaded ? (
          <>
            <div className={styles.backRow}>
              <Button asChild variant="outline">
                <Link to={SSL_LIST}>{t('outbound.ssl.backToList')}</Link>
              </Button>
            </div>
            {present && (
              <Text as="p" size="2" color="amber" role="status" className={styles.exists}>
                {t('outbound.ssl.exists')}
              </Text>
            )}
            <CertificateDetails certificate={loaded} />
            <div className={styles.trustWarning}>{t('outbound.ssl.trustWarning')}</div>
          </>
        ) : (
          <SettingsSection title={t('outbound.ssl.source')}>
            <RadioGroup.Root
              value={source}
              onValueChange={(value) => {
                setSource(value as Source);
                setProblem(null);
                setError(null);
              }}
              aria-label={t('outbound.ssl.source')}
              className={styles.source}
            >
              <div className={styles.option}>
                <RadioGroup.Item
                  value="server"
                  id="outbound-source-server"
                  aria-describedby="outbound-source-server-text"
                />
                <div className={styles.optionBody}>
                  <label htmlFor="outbound-source-server" className={styles.optionTitle}>
                    {t('outbound.ssl.fromServer')}
                  </label>
                  <span id="outbound-source-server-text" className={styles.optionText}>
                    {t('outbound.ssl.fromServerText')}
                  </span>
                  {source === 'server' && (
                    <div className={styles.optionInput}>
                      <SettingsFields>
                        <TextField
                          label={t('outbound.ssl.server')}
                          hint={t('outbound.ssl.serverHint')}
                          placeholder={t('outbound.ssl.serverPlaceholder')}
                          value={server}
                          onChange={(value) => {
                            setServer(value);
                            setProblem(null);
                          }}
                          error={problem ?? undefined}
                          required
                        />
                      </SettingsFields>
                    </div>
                  )}
                </div>
              </div>
              <div className={styles.option}>
                <RadioGroup.Item
                  value="pem"
                  id="outbound-source-pem"
                  aria-describedby="outbound-source-pem-text"
                />
                <div className={styles.optionBody}>
                  <label htmlFor="outbound-source-pem" className={styles.optionTitle}>
                    {t('outbound.ssl.fromPem')}
                  </label>
                  <span id="outbound-source-pem-text" className={styles.optionText}>
                    {t('outbound.ssl.fromPemText')}
                  </span>
                  {source === 'pem' && (
                    <div className={styles.optionInput}>
                      <PemField
                        value={pem}
                        onChange={(value) => {
                          setPem(value);
                          setProblem(null);
                        }}
                        error={problem}
                      />
                    </div>
                  )}
                </div>
              </div>
            </RadioGroup.Root>
          </SettingsSection>
        )}
        <FormError message={error} />
      </SettingsForm>
    </Page>
  );
}

function PemField({
  value,
  onChange,
  error,
}: {
  value: string;
  onChange(value: string): void;
  error: string | null;
}) {
  const t = useT();
  return (
    <div className={styles.optionBody}>
      <label htmlFor="outbound-pem" className={styles.inlineLabel}>
        {t('outbound.ssl.pem')}
        <span aria-hidden="true" className={styles.required}>
          {' *'}
        </span>
      </label>
      <Text as="p" size="1" color="gray" id="outbound-pem-hint" className={styles.inlineHint}>
        {t('outbound.ssl.pemHint')}
      </Text>
      <TextArea
        id="outbound-pem"
        value={value}
        rows={10}
        placeholder="-----BEGIN CERTIFICATE-----"
        style={{ fontFamily: 'var(--code-font-family)' }}
        aria-describedby="outbound-pem-hint"
        aria-invalid={error ? true : undefined}
        onChange={(event: FormEvent<HTMLTextAreaElement>) => onChange(event.currentTarget.value)}
      />
      {error && (
        <Text as="p" size="1" color="red" role="alert" mt="1">
          {error}
        </Text>
      )}
    </div>
  );
}
