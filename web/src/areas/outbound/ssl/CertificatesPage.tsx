import { Button } from '@radix-ui/themes';
import { Plus } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router';
import { useApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ErrorState, Page } from '../../../ui';
import {
  type ListColumn,
  ListTable,
  SearchBox,
  sortList,
  useCollator,
} from '../../security/shared/AdminList';
import list from '../../security/shared/AdminList.module.css';
import { type Certificate, certificateHref, SSL_ADD, SSL_DOCS, TRUSTSTORE, TRUSTSTORE_CREATE } from '../api';
import { AboutCard, DocsLink } from '../components';
import { certificateName, matchesCertificate } from '../model';
import styles from '../Outbound.module.css';

export default function CertificatesPage() {
  const t = useT();
  const navigate = useNavigate();
  const collator = useCollator();
  const canAdd = useCan(TRUSTSTORE_CREATE);
  const [params, setParams] = useSearchParams();
  const filter = params.get('q') ?? '';
  const certificates = useApi<Certificate[]>(TRUSTSTORE);
  const [sort, setSort] = useState<{ column: string; direction: 'asc' | 'desc' }>({
    column: 'name',
    direction: 'asc',
  });
  const columns: ListColumn<Certificate>[] = [
    {
      id: 'name',
      header: t('outbound.ssl.name'),
      width: '16%',
      cell: (certificate) => certificateName(certificate),
      sortValue: (certificate) => certificateName(certificate),
    },
    {
      id: 'issuedTo',
      header: t('outbound.ssl.issuedTo'),
      width: '15%',
      cell: (certificate) => certificate.subjectOrganization || '—',
      sortValue: (certificate) => certificate.subjectOrganization ?? '',
    },
    {
      id: 'issuedBy',
      header: t('outbound.ssl.issuedBy'),
      width: '15%',
      cell: (certificate) => certificate.issuerOrganization || '—',
      sortValue: (certificate) => certificate.issuerOrganization ?? '',
    },
    {
      id: 'fingerprint',
      header: t('outbound.ssl.fingerprint'),
      cell: (certificate) => <span className={styles.fingerprint}>{certificate.fingerprint}</span>,
      sortValue: (certificate) => certificate.fingerprint,
    },
  ];
  const column = columns.find((candidate) => candidate.id === sort.column);
  const rows = useMemo(
    () =>
      sortList(
        (certificates.data ?? []).filter((certificate) => matchesCertificate(certificate, filter)),
        column?.sortValue,
        sort.direction,
        collator,
      ),
    [certificates.data, filter, column, sort.direction, collator],
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
      <div className={styles.stack}>
        <div className={styles.filter}>
          <SearchBox value={filter} onChange={search} placeholder={t('outbound.ssl.filter')} />
        </div>
        {certificates.error ? (
          <ErrorState error={certificates.error} onRetry={() => void certificates.refetch()} />
        ) : (certificates.data ?? []).length === 0 && !certificates.isPending ? (
          <div className={styles.empty}>{t('outbound.ssl.empty')}</div>
        ) : (
          <ListTable
            plain
            chevron
            label={t('outbound.ssl.title')}
            columns={columns}
            rows={rows}
            loading={certificates.isPending}
            getRowId={(certificate) => certificate.id}
            rowLabel={(certificate) => t('outbound.ssl.open', { name: certificateName(certificate) })}
            onOpen={(certificate) => navigate(certificateHref(certificate.id))}
            sort={sort}
            onSort={(id) =>
              setSort(
                sort.column === id
                  ? { column: id, direction: sort.direction === 'asc' ? 'desc' : 'asc' }
                  : { column: id, direction: 'asc' },
              )
            }
            empty={<div className={styles.empty}>{t('outbound.ssl.noMatches', { filter })}</div>}
          />
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
