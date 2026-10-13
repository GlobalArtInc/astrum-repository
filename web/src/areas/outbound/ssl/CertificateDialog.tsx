import { Button, Dialog, Flex, Spinner, Text } from '@radix-ui/themes';
import { useState } from 'react';
import { api, useApi, useInvalidateApi, withQuery } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { describeError, useToast } from '../../../ui';
import { type Certificate, SSL, sendText, TRUSTSTORE, TRUSTSTORE_CREATE, TRUSTSTORE_DELETE } from '../api';
import { certificateName } from '../model';
import styles from '../Outbound.module.css';
import { CompactDetails } from './CertificateDetails';

function DialogContent({ host, port, onClose }: { host: string; port: number; onClose(): void }) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const canAdd = useCan(TRUSTSTORE_CREATE);
  const canRemove = useCan(TRUSTSTORE_DELETE);
  const certificate = useApi<Certificate>(withQuery(SSL, { host, port }), { staleTime: 0, gcTime: 0 });
  const trusted = useApi<Certificate[]>(TRUSTSTORE);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const present = certificate.data
    ? (trusted.data ?? []).some((known) => known.id === certificate.data?.id)
    : false;
  const add = async (found: Certificate) => {
    setBusy(true);
    setError(null);
    try {
      await sendText<Certificate>(TRUSTSTORE, found.pem);
      toast.success(t('outbound.ssl.added', { name: certificateName(found) }));
      await invalidate(TRUSTSTORE);
    } catch (failure) {
      setError(`${t('outbound.ssl.dialog.addFailed')}: ${describeError(t, failure)}`);
    } finally {
      setBusy(false);
    }
  };
  const remove = async (found: Certificate) => {
    setBusy(true);
    setError(null);
    try {
      await api.delete(`${TRUSTSTORE}/${encodeURIComponent(found.id)}`);
      toast.success(t('outbound.ssl.removed', { name: certificateName(found) }));
      await invalidate(TRUSTSTORE);
    } catch (failure) {
      setError(`${t('outbound.ssl.dialog.removeFailed')}: ${describeError(t, failure)}`);
    } finally {
      setBusy(false);
    }
  };
  const found = certificate.data;
  return (
    <>
      <Dialog.Title>{t('outbound.ssl.dialog.title')}</Dialog.Title>
      <Dialog.Description size="2" color="gray">
        {t('outbound.ssl.dialog.server', { server: `${host}:${port}` })}
      </Dialog.Description>
      <div className={styles.dialogBody}>
        {certificate.isPending ? (
          <Flex align="center" gap="3" py="5" role="status">
            <Spinner />
            <Text size="2">{t('outbound.ssl.dialog.retrieving')}</Text>
          </Flex>
        ) : certificate.error ? (
          <Text as="p" size="2" color="red" role="alert" mt="4">
            {t('outbound.ssl.dialog.fetchFailed')}: {describeError(t, certificate.error)}
          </Text>
        ) : found ? (
          <>
            <CompactDetails certificate={found} />
            {!present && <div className={styles.dialogWarning}>{t('outbound.ssl.dialog.untrusted')}</div>}
          </>
        ) : null}
        {error && (
          <Text as="p" size="2" color="red" role="alert">
            {error}
          </Text>
        )}
      </div>
      <Flex gap="3" mt="5" justify="end" wrap="wrap">
        {found && present && canRemove && (
          <Button color="red" variant="soft" loading={busy} onClick={() => void remove(found)}>
            {t('outbound.ssl.dialog.remove')}
          </Button>
        )}
        {found && !present && canAdd && (
          <Button loading={busy} onClick={() => void add(found)}>
            {t('outbound.ssl.dialog.add')}
          </Button>
        )}
        <Button variant="outline" color="gray" onClick={onClose}>
          {t('outbound.ssl.dialog.close')}
        </Button>
      </Flex>
    </>
  );
}

export function CertificateDialog({
  open,
  onOpenChange,
  host,
  port,
}: {
  open: boolean;
  onOpenChange(open: boolean): void;
  host: string;
  port: number;
}) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Content maxWidth="640px">
        {open && <DialogContent host={host} port={port} onClose={() => onOpenChange(false)} />}
      </Dialog.Content>
    </Dialog.Root>
  );
}
