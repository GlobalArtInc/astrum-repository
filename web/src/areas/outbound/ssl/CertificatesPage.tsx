import { Button } from '@radix-ui/themes';
import { ChevronDown, ChevronRight, ChevronsUpDown, ChevronUp, Plus, Search } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router';
import { useApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ErrorState, Page } from '../../../ui';
import { sortList, useCollator } from '../../security/shared/AdminList';
import list from '../../security/shared/AdminList.module.css';
import { type Certificate, certificateHref, SSL_ADD, SSL_DOCS, TRUSTSTORE, TRUSTSTORE_CREATE } from '../api';
import { AboutCard, DocsLink } from '../components';
import { certificateName, expiry, matchesCertificate } from '../model';
import styles from '../Outbound.module.css';

type Column = 'name' | 'issuedTo' | 'issuedBy' | 'fingerprint';

const SORT_VALUES: Record<Column, (certificate: Certificate) => string> = {
  name: (certificate) => certificateName(certificate),
  issuedTo: (certificate) => certificate.subjectOrganization ?? '',
  issuedBy: (certificate) => certificate.issuerOrganization ?? '',
  fingerprint: (certificate) => certificate.fingerprint,
};

export default function CertificatesPage() {
  const t = useT();
  const navigate = useNavigate();
  const collator = useCollator();
  const canAdd = useCan(TRUSTSTORE_CREATE);
  const [params, setParams] = useSearchParams();
  const filter = params.get('q') ?? '';
  const certificates = useApi<Certificate[]>(TRUSTSTORE);
  const [sort, setSort] = useState<{ column: Column; direction: 'asc' | 'desc' }>({
    column: 'name',
    direction: 'asc',
  });
  const rows = useMemo(
    () =>
      sortList(
        (certificates.data ?? []).filter((certificate) => matchesCertificate(certificate, filter)),
        SORT_VALUES[sort.column],
        sort.direction,
        collator,
      ),
    [certificates.data, filter, sort, collator],
  );
  const search = (value: string) =>
    setParams(
      (previous) => {
        const next = new URLSearchParams(previous);
        if (value) {
          next.set('q', value);
        } else {
          next.delete('q');
        }
        return next;
      },
      { replace: true },
    );
  const header = (column: Column, label: string) => {
    const sorted = sort.column === column ? sort.direction : undefined;
    return (
      <th
        className={styles.sslTh}
        aria-sort={sorted === 'asc' ? 'ascending' : sorted === 'desc' ? 'descending' : undefined}
      >
        <button
          type="button"
          className={styles.sslSort}
          onClick={() =>
            setSort(
              sorted
                ? { column, direction: sorted === 'asc' ? 'desc' : 'asc' }
                : { column, direction: 'asc' },
            )
          }
        >
          {label}
          {sorted === 'asc' && <ChevronUp size={14} aria-hidden="true" className={styles.sslSortIcon} />}
          {sorted === 'desc' && <ChevronDown size={14} aria-hidden="true" className={styles.sslSortIcon} />}
          {!sorted && <ChevronsUpDown size={14} aria-hidden="true" className={styles.sslSortIcon} />}
        </button>
      </th>
    );
  };
  const rowClass = (certificate: Certificate) => {
    const state = expiry(certificate);
    return state === 'expired'
      ? `${styles.sslRow} ${styles.sslExpired}`
      : state === 'expiring'
        ? `${styles.sslRow} ${styles.sslExpiring}`
        : styles.sslRow;
  };
  const all = certificates.data ?? [];
  return (
    <Page
      className={styles.page}
      title={t('outbound.ssl.title')}
      description={t('outbound.ssl.description')}
      breadcrumbs={[
        { label: t('outbound.crumbs.settings'), to: '/settings' },
        { label: t('outbound.ssl.title') },
      ]}
      actions={
        canAdd && (
          <Button asChild size="3" className={list.bigButton}>
            <Link to={SSL_ADD}>
              <Plus size={16} aria-hidden="true" />
              {t('outbound.ssl.add')}
            </Link>
          </Button>
        )
      }
    >
      <div className={styles.sslList}>
        <div className={styles.sslSearch}>
          <Search size={16} aria-hidden="true" className={styles.sslSearchIcon} />
          <input
            type="text"
            className={styles.sslSearchInput}
            value={filter}
            placeholder={t('outbound.ssl.filter')}
            aria-label={t('outbound.ssl.filter')}
            onChange={(event) => search(event.target.value)}
          />
        </div>
        {certificates.error ? (
          <ErrorState error={certificates.error} onRetry={() => void certificates.refetch()} />
        ) : certificates.isPending ? (
          <div className={styles.sslEmpty} aria-busy="true" />
        ) : all.length === 0 ? (
          <div className={styles.sslEmpty}>{t('outbound.ssl.empty')}</div>
        ) : rows.length === 0 ? (
          <div className={styles.sslEmpty}>{t('outbound.ssl.noMatches', { filter })}</div>
        ) : (
          <div className={styles.sslTableCard}>
            <table className={styles.sslTable} aria-label={t('outbound.ssl.title')}>
              <thead>
                <tr>
                  {header('name', t('outbound.ssl.name'))}
                  {header('issuedTo', t('outbound.ssl.issuedTo'))}
                  {header('issuedBy', t('outbound.ssl.issuedBy'))}
                  {header('fingerprint', t('outbound.ssl.fingerprint'))}
                  <th className={`${styles.sslTh} ${styles.sslActionHead}`}>
                    <span className="visually-hidden">{t('outbound.ssl.title')}</span>
                  </th>
                </tr>
              </thead>
              <tbody>
                {rows.map((certificate) => {
                  const name = certificateName(certificate);
                  const open = () => navigate(certificateHref(certificate.id));
                  return (
                    <tr
                      key={certificate.id}
                      className={rowClass(certificate)}
                      tabIndex={0}
                      aria-label={t('outbound.ssl.open', { name })}
                      onClick={open}
                      onKeyDown={(event) => {
                        if (event.key === 'Enter' && event.target === event.currentTarget) {
                          open();
                        }
                      }}
                    >
                      <td className={styles.sslTd}>{name}</td>
                      <td className={styles.sslTd}>{certificate.subjectOrganization || '—'}</td>
                      <td className={styles.sslTd}>{certificate.issuerOrganization || '—'}</td>
                      <td className={styles.sslTd}>
                        <span className={styles.fingerprint}>{certificate.fingerprint}</span>
                      </td>
                      <td className={`${styles.sslTd} ${styles.sslAction}`}>
                        <ChevronRight size={16} aria-hidden="true" className={styles.sslOpen} />
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
        <AboutCard accent title={t('outbound.ssl.what')}>
          <p className={styles.aboutText}>
            {t.rich('outbound.ssl.whatText', {
              link: (
                <DocsLink key="docs" href={SSL_DOCS}>
                  {t('outbound.common.docs')}
                </DocsLink>
              ),
            })}
          </p>
        </AboutCard>
      </div>
    </Page>
  );
}
