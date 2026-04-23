import { Button, Flex, Spinner } from '@radix-ui/themes';
import { Trash2 } from 'lucide-react';
import { useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { api, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ConfirmDialog, EmptyState, ErrorState, Page, useToast } from '../../../ui';
import { SettingsForm, settingsButton } from '../../../ui/SettingsForm';
import { type Certificate, SSL_LIST, TRUSTSTORE, TRUSTSTORE_DELETE } from '../api';
import { certificateName } from '../model';
import styles from '../Outbound.module.css';
import { CertificateDetails } from './CertificateDetails';

export default function CertificatePage() {
  const t = useT();
  const { id = '' } = useParams();
  const navigate = useNavigate();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const canDelete = useCan(TRUSTSTORE_DELETE);
  const certificates = useApi<Certificate[]>(TRUSTSTORE);
  const [deleting, setDeleting] = useState(false);
  const certificate = certificates.data?.find((candidate) => candidate.id === id);
  const name = certificate ? certificateName(certificate) : id;
  const remove = async () => {
    if (!certificate) {
      return;
    }
    await api.delete(`${TRUSTSTORE}/${encodeURIComponent(certificate.id)}`);
    toast.success(t('outbound.ssl.removed', { name }));
    await invalidate(TRUSTSTORE);
    navigate(SSL_LIST);
  };
  return (
    <Page
      className={styles.page}
      title={t('outbound.ssl.certificateTitle', { name })}
      breadcrumbs={[
        { label: t('outbound.crumbs.settings'), to: '/settings' },
        { label: t('outbound.ssl.title'), to: SSL_LIST },
        { label: name },
      ]}
    >
      {certificates.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : certificates.error ? (
        <ErrorState error={certificates.error} onRetry={() => void certificates.refetch()} />
      ) : !certificate ? (
        <EmptyState title={t('outbound.ssl.notFound')} />
      ) : (
        <SettingsForm
          wide
          label={t('outbound.ssl.certificateTitle', { name })}
          start={
            canDelete && (
              <Button
                type="button"
                size="3"
                color="red"
                className={`${settingsButton} ${styles.iconGap}`}
                onClick={() => setDeleting(true)}
              >
                <Trash2 size={16} aria-hidden="true" />
                {t('outbound.ssl.delete')}
              </Button>
            )
          }
          end={
            <Button asChild variant="outline">
              <Link to={SSL_LIST}>{t('outbound.ssl.backToList')}</Link>
            </Button>
          }
        >
          <CertificateDetails certificate={certificate} />
          <ConfirmDialog
            open={deleting}
            onOpenChange={setDeleting}
            onConfirm={remove}
            danger
            title={t('outbound.ssl.deleteTitle')}
            description={t('outbound.ssl.deleteText', { name })}
            cancelLabel={t('outbound.common.cancel')}
            confirmLabel={t('outbound.ssl.delete')}
          />
        </SettingsForm>
      )}
    </Page>
  );
}
