import { Button, Heading, Skeleton, Text, TextField } from '@radix-ui/themes';
import { useQuery } from '@tanstack/react-query';
import { type ReactNode, type RefObject, useEffect, useRef, useState } from 'react';
import { apiKey, useApi } from '../../../api';
import { useT } from '../../../i18n';
import { type Column, DataTable, ErrorState, StatusBadge } from '../../../ui';
import { pageCount } from '../Pager';
import {
  fetchSbomPage,
  type SbomPackage,
  type SbomReference,
  SbomRequestError,
  type Signature,
  type SupplyChain,
} from './api';
import {
  FILTER_DELAY_MS,
  licencesText,
  packageKey,
  SBOM_PAGE_SIZE,
  sbomFormatKey,
  sbomProblem,
  sbomUrl,
  shortDigest,
  signatureSchemeKey,
  signatureStateKey,
  signatureTone,
  signedBy,
  supplyChainUrl,
  truncatePurl,
} from './model';
import styles from './SupplyChain.module.css';

interface NoticeProps {
  children: ReactNode;
}

interface SignatureLineProps {
  signature: Signature;
}

interface SignaturesCardProps {
  data: SupplyChain;
}

interface BusyNoticeProps {
  retryAfter: number | null;
  onRetry: () => void;
}

interface SbomFailureProps {
  error: unknown;
  updatedAt: number;
  onRetry: () => void;
}

export interface SupplyChainProps {
  repository: string;
  image: string;
  reference: string;
}

interface SbomPackagesProps {
  repository: string;
  image: string;
  digest: string;
}

interface SbomCardProps {
  repository: string;
  image: string;
  sboms: SbomReference[];
}

interface SupplyChainCardsProps {
  repository: string;
  image: string;
  data: SupplyChain;
}

interface PackageRow {
  key: string;
  item: SbomPackage;
}

export function useOnceVisible<T extends Element>(): [RefObject<T | null>, boolean] {
  const ref = useRef<T | null>(null);
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    const element = ref.current;
    if (visible || element === null) {
      return;
    }
    if (typeof IntersectionObserver === 'undefined') {
      setVisible(true);
      return;
    }
    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting)) {
        setVisible(true);
        observer.disconnect();
      }
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, [visible]);
  return [ref, visible];
}

function Notice({ children }: NoticeProps) {
  return (
    <Text as="p" size="2" color="gray">
      {children}
    </Text>
  );
}

function SignatureLine({ signature }: SignatureLineProps) {
  const t = useT();
  const label =
    signature.state === 'verified' && signature.signer
      ? t('browse.docker.state.verifiedBy', { signer: signature.signer })
      : t(signatureStateKey(signature.state));
  return (
    <li className={styles.row}>
      <Text size="2">{t(signatureSchemeKey(signature.scheme))}</Text>
      <StatusBadge tone={signatureTone(signature.state)}>{label}</StatusBadge>
      <code className={styles.digest} title={signature.digest}>
        {shortDigest(signature.digest)}
      </code>
    </li>
  );
}

function SignaturesCard({ data }: SignaturesCardProps) {
  const t = useT();
  return (
    <section className={styles.card} aria-labelledby="docker-signatures">
      <Heading as="h2" size="5" weight="bold" id="docker-signatures" className={styles.cardTitle}>
        {t('browse.docker.signatures.title')}
      </Heading>
      {data.children.length > 0 && (
        <Notice>{t('browse.docker.signatures.index', { count: data.children.length })}</Notice>
      )}
      {data.signatures.length === 0 ? (
        <Notice>{t('browse.docker.signatures.empty')}</Notice>
      ) : (
        <ul className={styles.list}>
          {data.signatures.map((signature) => (
            <SignatureLine key={`${signature.scheme}:${signature.digest}`} signature={signature} />
          ))}
        </ul>
      )}
      {data.attestations.length > 0 && (
        <div className={styles.list}>
          <Text as="p" size="2" weight="medium">
            {t('browse.docker.attestations.count', { count: data.attestations.length })}
          </Text>
          <ul className={styles.list}>
            {data.attestations.map((attestation) => (
              <li key={attestation.digest}>
                <code className={styles.digest} title={attestation.digest}>
                  {shortDigest(attestation.digest)}
                </code>
              </li>
            ))}
          </ul>
        </div>
      )}
    </section>
  );
}

function BusyNotice({ retryAfter, onRetry }: BusyNoticeProps) {
  const t = useT();
  const [waiting, setWaiting] = useState(retryAfter ?? 0);
  useEffect(() => {
    const timer = waiting > 0 ? window.setTimeout(() => setWaiting(waiting - 1), 1000) : undefined;
    return () => window.clearTimeout(timer);
  }, [waiting]);
  return (
    <div className={styles.notice}>
      <Text as="p" size="2" color="gray">
        {waiting > 0 ? t('browse.docker.sbom.busyWait', { seconds: waiting }) : t('browse.docker.sbom.busy')}
      </Text>
      <Button variant="soft" color="gray" disabled={waiting > 0} onClick={onRetry}>
        {t('browse.docker.sbom.retry')}
      </Button>
    </div>
  );
}

function SbomFailure({ error, updatedAt, onRetry }: SbomFailureProps) {
  const t = useT();
  if (!(error instanceof SbomRequestError)) {
    return <ErrorState error={error} onRetry={onRetry} />;
  }
  switch (sbomProblem(error.status)) {
    case 'tooLarge':
      return <Notice>{t('browse.docker.sbom.tooLarge')}</Notice>;
    case 'unreadable':
      return <Notice>{t('browse.docker.sbom.unreadable')}</Notice>;
    case 'notFound':
      return <Notice>{t('browse.docker.sbom.notFound')}</Notice>;
    case 'badRequest':
      return <Notice>{t('browse.docker.sbom.badRequest')}</Notice>;
    case 'busy':
      return <BusyNotice key={updatedAt} retryAfter={error.retryAfter} onRetry={onRetry} />;
    default:
      return <ErrorState error={error} onRetry={onRetry} />;
  }
}

function SbomPackages({ repository, image, digest }: SbomPackagesProps) {
  const t = useT();
  const [filter, setFilter] = useState('');
  const [applied, setApplied] = useState('');
  const [page, setPage] = useState(1);
  useEffect(() => {
    const next = filter.trim();
    const timer =
      next === applied
        ? undefined
        : window.setTimeout(() => {
            setApplied(next);
            setPage(1);
          }, FILTER_DELAY_MS);
    return () => window.clearTimeout(timer);
  }, [filter, applied]);
  const url = sbomUrl(repository, image, digest, page, applied);
  const query = useQuery({
    queryKey: apiKey(url),
    queryFn: ({ signal }) => fetchSbomPage(url, signal),
    retry: false,
  });
  const columns: Column<PackageRow>[] = [
    { id: 'name', header: t('browse.docker.sbom.column.name'), cell: (row) => row.item.name, primary: true },
    { id: 'version', header: t('browse.docker.sbom.column.version'), cell: (row) => row.item.version },
    {
      id: 'licences',
      header: t('browse.docker.sbom.column.licences'),
      cell: (row) => (
        <span className={styles.clip} title={licencesText(row.item.licenses)}>
          {licencesText(row.item.licenses)}
        </span>
      ),
    },
    { id: 'ecosystem', header: t('browse.docker.sbom.column.ecosystem'), cell: (row) => row.item.ecosystem },
    {
      id: 'purl',
      header: t('browse.docker.sbom.column.purl'),
      cell: (row) => (
        <code className={`${styles.clip} ${styles.purl}`} title={row.item.purl}>
          {truncatePurl(row.item.purl)}
        </code>
      ),
    },
  ];
  const filterField = (
    <TextField.Root
      value={filter}
      onChange={(event) => setFilter(event.target.value)}
      placeholder={t('browse.docker.sbom.filterPlaceholder')}
      aria-label={t('browse.docker.sbom.filter')}
    />
  );
  if (query.isPending) {
    return (
      <div className={styles.sbom}>
        {filterField}
        <Skeleton height="200px" />
      </div>
    );
  }
  if (query.isError) {
    return (
      <div className={styles.sbom}>
        {filterField}
        <SbomFailure error={query.error} updatedAt={query.errorUpdatedAt} onRetry={() => query.refetch()} />
      </div>
    );
  }
  const data = query.data;
  const rows: PackageRow[] = data.packages.map((item, index) => ({
    key: packageKey(page, index, item),
    item,
  }));
  const pages = pageCount(data.total, SBOM_PAGE_SIZE);
  return (
    <div className={styles.sbom}>
      {filterField}
      <Text as="p" size="2" color="gray">
        {t('browse.docker.sbom.total', { count: data.total })}
      </Text>
      <DataTable
        label={t('browse.docker.sbom.table')}
        columns={columns}
        rows={rows}
        getRowId={(row) => row.key}
        empty={
          <Notice>
            {applied === '' ? t('browse.docker.sbom.noPackages') : t('browse.docker.sbom.noMatch')}
          </Notice>
        }
      />
      {data.total > SBOM_PAGE_SIZE && (
        <nav className={styles.pager} aria-label={t('browse.docker.sbom.pages')}>
          <Button variant="soft" color="gray" disabled={page <= 1} onClick={() => setPage(page - 1)}>
            {t('browse.docker.sbom.previous')}
          </Button>
          <Text size="2">{t('browse.docker.sbom.page', { page, pages })}</Text>
          <Button variant="soft" color="gray" disabled={page >= pages} onClick={() => setPage(page + 1)}>
            {t('browse.docker.sbom.next')}
          </Button>
        </nav>
      )}
    </div>
  );
}

function SbomCard({ repository, image, sboms }: SbomCardProps) {
  const t = useT();
  const [chosen, setChosen] = useState<string | null>(null);
  const selected = sboms.find((sbom) => sbom.digest === chosen) ?? sboms[0];
  return (
    <section className={styles.card} aria-labelledby="docker-sbom">
      <Heading as="h2" size="5" weight="bold" id="docker-sbom" className={styles.cardTitle}>
        {t('browse.docker.sbom.title')}
      </Heading>
      {selected === undefined ? (
        <Notice>{t('browse.docker.sbom.empty')}</Notice>
      ) : (
        <>
          <div className={styles.choices} role="group" aria-label={t('browse.docker.sbom.choose')}>
            {sboms.map((sbom) => (
              <Button
                key={sbom.digest}
                variant={sbom.digest === selected.digest ? 'solid' : 'soft'}
                color="gray"
                aria-pressed={sbom.digest === selected.digest}
                onClick={() => setChosen(sbom.digest)}
              >
                {t(sbomFormatKey(sbom.format))}
                <code title={sbom.digest}>{shortDigest(sbom.digest)}</code>
              </Button>
            ))}
          </div>
          <SbomPackages
            key={selected.digest}
            repository={repository}
            image={image}
            digest={selected.digest}
          />
        </>
      )}
    </section>
  );
}

function SupplyChainCards({ repository, image, data }: SupplyChainCardsProps) {
  return (
    <>
      <SignaturesCard data={data} />
      <SbomCard repository={repository} image={image} sboms={data.sboms} />
    </>
  );
}

export function ImageSupplyChain({ repository, image, reference }: SupplyChainProps) {
  const [gate, visible] = useOnceVisible<HTMLDivElement>();
  const query = useApi<SupplyChain>(visible ? supplyChainUrl(repository, image, reference) : null);
  let body: ReactNode = <Skeleton height="160px" />;
  if (visible && query.isError) {
    body = <ErrorState error={query.error} onRetry={() => query.refetch()} />;
  } else if (visible && query.data) {
    body = <SupplyChainCards repository={repository} image={image} data={query.data} />;
  }
  return (
    <div ref={gate} className={styles.panels}>
      {body}
    </div>
  );
}

export function SigningChip({ repository, image, reference }: SupplyChainProps) {
  const t = useT();
  const query = useApi<SupplyChain>(supplyChainUrl(repository, image, reference));
  if (!query.data) {
    return null;
  }
  const signer = signedBy(query.data.signatures);
  if (signer === null) {
    return <StatusBadge tone="neutral">{t('browse.docker.chip.notSigned')}</StatusBadge>;
  }
  return <StatusBadge tone="success">{t('browse.docker.chip.signedBy', { signer })}</StatusBadge>;
}
