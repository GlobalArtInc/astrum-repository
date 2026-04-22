import { Button, Text } from '@radix-ui/themes';
import { type FormEvent, useEffect, useState } from 'react';
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
import { Alert, CertificateDetails } from './CertificateDetails';

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
  const [leaveTo, setLeaveTo] = useState<string | null>(null);
  const present = loaded ? (trusted.data ?? []).some((known) => known.id === loaded.id) : false;
  const ready = source === 'pem' || server.trim() !== '';
  useEffect(() => {
    if (leaveTo !== null) {
      navigate(leaveTo);
    }
  }, [leaveTo, navigate]);
  const choose = (next: Source) => {
    setSource(next);
    setProblem(null);
    setError(null);
  };

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
      setLeaveTo(certificateHref(created?.id ?? loaded.id));
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
      setLeaveTo(SSL_LIST);
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
        <UnsavedChangesGuard when={loaded !== null && leaveTo === null} />
        {loaded ? (
          <>
            <div className={styles.innerBar}>
              <Button asChild variant="outline">
                <Link to={SSL_LIST}>{t('outbound.ssl.backToList')}</Link>
              </Button>
            </div>
            <div className={styles.innerContent}>
              {present && (
                <div className={styles.alerts}>
                  <Alert>{t('outbound.ssl.exists')}</Alert>
                </div>
              )}
              <CertificateDetails certificate={loaded} />
              <div className={styles.trustWarning}>
                <Alert>{t('outbound.ssl.trustWarning')}</Alert>
              </div>
            </div>
          </>
        ) : (
          <SettingsSection title={t('outbound.ssl.source')}>
            <div className={styles.sourceOptions} role="radiogroup" aria-label={t('outbound.ssl.source')}>
              <div className={styles.radioGroup}>
                <label className={styles.radioLabel}>
                  <input
                    type="radio"
                    name="outbound-source"
                    value="server"
                    className={styles.radioInput}
                    checked={source === 'server'}
                    aria-labelledby="outbound-source-server-title"
                    aria-describedby="outbound-source-server-text"
                    onChange={() => choose('server')}
                  />
                  <span className={styles.radioContent}>
                    <span id="outbound-source-server-title" className={styles.radioTitle}>
                      {t('outbound.ssl.fromServer')}
                    </span>
                    <span id="outbound-source-server-text" className={styles.radioText}>
                      {t('outbound.ssl.fromServerText')}
                    </span>
                  </span>
                </label>
                {source === 'server' && (
                  <div className={styles.radioField}>
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
              <div className={styles.radioGroup}>
                <label className={styles.radioLabel}>
                  <input
                    type="radio"
                    name="outbound-source"
                    value="pem"
                    className={styles.radioInput}
                    checked={source === 'pem'}
                    aria-labelledby="outbound-source-pem-title"
                    aria-describedby="outbound-source-pem-text"
                    onChange={() => choose('pem')}
                  />
                  <span className={styles.radioContent}>
                    <span id="outbound-source-pem-title" className={styles.radioTitle}>
                      {t('outbound.ssl.fromPem')}
                    </span>
                    <span id="outbound-source-pem-text" className={styles.radioText}>
                      {t('outbound.ssl.fromPemText')}
                    </span>
                  </span>
                </label>
                {source === 'pem' && (
                  <div className={styles.radioField}>
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
    <div className={styles.pemField}>
      <label htmlFor="outbound-pem" className={styles.pemLabel}>
        {t('outbound.ssl.pem')}
        <span aria-hidden="true" className={styles.required}>
          *
        </span>
      </label>
      <Text as="p" size="1" id="outbound-pem-hint" className={styles.pemHint}>
        {t('outbound.ssl.pemHint')}
      </Text>
      <textarea
        id="outbound-pem"
        className={styles.pemInput}
        value={value}
        rows={8}
        placeholder={'-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----'}
        aria-describedby="outbound-pem-hint"
        aria-invalid={error ? true : undefined}
        spellCheck={false}
        onChange={(event: FormEvent<HTMLTextAreaElement>) => onChange(event.currentTarget.value)}
      />
      {error && (
        <Text as="p" size="1" color="red" role="alert">
          {error}
        </Text>
      )}
    </div>
  );
}
