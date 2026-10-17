import { TriangleAlert } from 'lucide-react';
import { type ReactNode, useId } from 'react';
import { useI18n, useT } from '../../../i18n';
import type { Certificate } from '../api';
import { expiry } from '../model';
import styles from '../Outbound.module.css';

export function formatMoment(language: string, value: number): string {
  return new Intl.DateTimeFormat(language, {
    year: 'numeric',
    month: 'numeric',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    second: '2-digit',
  }).format(new Date(value));
}

function Rows({ rows }: { rows: Array<{ label: string; value: string | null; mono?: boolean }> }) {
  return (
    <dl className={styles.rows}>
      {rows.map((row) => (
        <div key={row.label} className={styles.row}>
          <dt>{row.label}</dt>
          <dd className={row.mono ? styles.mono : undefined}>{row.value?.trim() ? row.value : '—'}</dd>
        </div>
      ))}
    </dl>
  );
}

function InfoRows({ rows }: { rows: Array<{ label: string; value: string | null; mono?: boolean }> }) {
  return (
    <dl className={styles.info}>
      {rows.map((row) => (
        <div key={row.label} className={styles.infoRow}>
          <dt className={styles.infoLabel}>{row.label}</dt>
          <dd className={row.mono ? styles.infoFingerprint : styles.infoValue}>
            {row.value?.trim() ? row.value : '—'}
          </dd>
        </div>
      ))}
    </dl>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  const id = useId();
  return (
    <section className={styles.detailsSection} aria-labelledby={id}>
      <h2 id={id} className={styles.detailsTitle}>
        {title}
      </h2>
      {children}
      <hr className={styles.detailsSeparator} />
    </section>
  );
}

export function Alert({ tone = 'warning', children }: { tone?: 'warning' | 'error'; children: ReactNode }) {
  return (
    <div className={tone === 'error' ? `${styles.alert} ${styles.alertError}` : styles.alert} role="status">
      <TriangleAlert size={18} aria-hidden="true" className={styles.alertIcon} />
      <div className={styles.alertContent}>{children}</div>
    </div>
  );
}

export function ExpiryWarning({ certificate }: { certificate: Pick<Certificate, 'expiresOn'> }) {
  const t = useT();
  const state = expiry(certificate);
  if (state === 'valid') {
    return null;
  }
  return (
    <div className={styles.alerts}>
      <Alert tone={state === 'expired' ? 'error' : 'warning'}>
        <span className={styles.alertInline}>
          <TriangleAlert size={16} aria-hidden="true" />
          {state === 'expired' ? t('outbound.ssl.expired') : t('outbound.ssl.expiring')}
        </span>
      </Alert>
    </div>
  );
}

export function CertificateDetails({ certificate }: { certificate: Certificate }) {
  const t = useT();
  const { language } = useI18n();
  return (
    <div className={styles.details}>
      <ExpiryWarning certificate={certificate} />
      <Section title={t('outbound.ssl.information')}>
        <InfoRows
          rows={[
            { label: t('outbound.ssl.fingerprint'), value: certificate.fingerprint, mono: true },
            { label: t('outbound.ssl.validUntil'), value: formatMoment(language, certificate.expiresOn) },
            { label: t('outbound.ssl.issuedOn'), value: formatMoment(language, certificate.issuedOn) },
          ]}
        />
      </Section>
      <Section title={t('outbound.ssl.subject')}>
        <InfoRows
          rows={[
            { label: t('outbound.ssl.commonName'), value: certificate.subjectCommonName },
            { label: t('outbound.ssl.organization'), value: certificate.subjectOrganization },
            { label: t('outbound.ssl.unit'), value: certificate.subjectOrganizationalUnit },
          ]}
        />
      </Section>
      <Section title={t('outbound.ssl.issuer')}>
        <InfoRows
          rows={[
            { label: t('outbound.ssl.commonName'), value: certificate.issuerCommonName },
            { label: t('outbound.ssl.organization'), value: certificate.issuerOrganization },
            { label: t('outbound.ssl.unit'), value: certificate.issuerOrganizationalUnit },
          ]}
        />
      </Section>
    </div>
  );
}

export function CompactDetails({ certificate }: { certificate: Certificate }) {
  const t = useT();
  const { language } = useI18n();
  return (
    <div className={styles.dialogRows}>
      <Rows
        rows={[
          { label: t('outbound.ssl.commonName'), value: certificate.subjectCommonName },
          { label: t('outbound.ssl.organization'), value: certificate.subjectOrganization },
          { label: t('outbound.ssl.unit'), value: certificate.subjectOrganizationalUnit },
          { label: t('outbound.ssl.issuedOn'), value: formatMoment(language, certificate.issuedOn) },
          { label: t('outbound.ssl.validUntil'), value: formatMoment(language, certificate.expiresOn) },
          { label: t('outbound.ssl.issuerCommonName'), value: certificate.issuerCommonName },
          { label: t('outbound.ssl.issuerOrganization'), value: certificate.issuerOrganization },
          { label: t('outbound.ssl.issuerUnit'), value: certificate.issuerOrganizationalUnit },
          { label: t('outbound.ssl.fingerprint'), value: certificate.fingerprint, mono: true },
        ]}
      />
    </div>
  );
}
