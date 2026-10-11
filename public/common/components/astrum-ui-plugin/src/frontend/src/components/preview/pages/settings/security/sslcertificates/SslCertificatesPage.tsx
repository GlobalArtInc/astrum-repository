
import React, { useState, useEffect, useCallback } from 'react';
import { Box, } from '@radix-ui/themes';
import { Plus, } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { useToast, PageHeader } from '../../../../shared';
import { SettingsButton, SettingsAlert, ConfirmDialog } from '../../../../shared/form';
import { SslCertificatesList } from './SslCertificatesList';
import { SslCertificatesDetail } from './SslCertificatesDetail';
import { SslCertificatesAddForm } from './SslCertificatesAddForm';
import { useSslCertificatesApi } from './useSslCertificatesApi';
import { SslCertificate, AddCertificateFormData } from './types';

import './SslCertificatesPage.scss';
import SslCertificatesStrings from '../../../../constants/pages/settings/security/SslCertificatesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

const navigateTo = (path: string) => {
  window.location.hash = path;
};

type ViewMode = 'list' | 'create' | 'detail';

/**
 * SslCertificatesPage - Main SSL Certificates management page for Preview UI
 * 
 * Displays certificate list with search/filter, and allows adding and deleting certificates.
 */
export function SslCertificatesPage() {
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [selectedCertificateId, setSelectedCertificateId] = useState<string | null>(null);
  const [certificate, setCertificate] = useState<SslCertificate | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  // Toast notifications (app-level provider)
  const toast = useToast();

  const {
    loading,
    error,
    setError,
    fetchCertificates,
    addCertificate,
    deleteCertificate,
  } = useSslCertificatesApi();

  // Permissions load asynchronously (ASTRUM-53199), so checkPermission() can legitimately report
  // false on the first paint. Nothing else re-renders this page afterwards, so without
  // re-evaluating once they arrive the create/delete affordances would stay hidden for the whole
  // session even for an admin (ASTRUM-54265).
  const [{ canCreate, canDelete }, setPermissions] = useState(() => ({
    canCreate: ExtJS.checkPermission('astrum:ssl-truststore:create'),
    canDelete: ExtJS.checkPermission('astrum:ssl-truststore:delete'),
  }));

  useEffect(() => {
    let cancelled = false;

    ExtJS.waitForPermissions()
      .then(() => {
        if (cancelled) return;
        setPermissions({
          canCreate: ExtJS.checkPermission('astrum:ssl-truststore:create'),
          canDelete: ExtJS.checkPermission('astrum:ssl-truststore:delete'),
        });
      })
      // On timeout keep the initial evaluation rather than blanking the page.
      .catch(() => {});

    return () => {
      cancelled = true;
    };
  }, []);

  // Load certificate details when selected
  useEffect(() => {
    if (selectedCertificateId && viewMode === 'detail') {
      fetchCertificates()
        .then((certificates) => {
          const found = certificates.find((c) => c.id === selectedCertificateId);
          if (found) {
            setCertificate(found);
          } else {
            setError(SslCertificatesStrings.PAGE.CERTIFICATE_NOT_FOUND);
            setViewMode('list');
          }
        })
        .catch((err) => {
          setError(err.message);
          setViewMode('list');
        });
    }
  }, [selectedCertificateId, viewMode, fetchCertificates, setError]);

  const handleSelectCertificate = useCallback((certificateId: string) => {
    setSelectedCertificateId(certificateId);
    setCertificate(null);
    setError(null);
    setViewMode('detail');
  }, [setError]);

  const handleCreate = useCallback(() => {
    setSelectedCertificateId(null);
    setCertificate(null);
    setError(null);
    setViewMode('create');
  }, [setError]);

  const handleBack = useCallback(() => {
    setSelectedCertificateId(null);
    setCertificate(null);
    setError(null);
    setViewMode('list');
  }, [setError]);

  const handleSave = useCallback(async (data: AddCertificateFormData) => {
    try {
      const result = await addCertificate(data.pemContent);
      toast.success(SslCertificatesStrings.PAGE.SSL_CERTIFICATE_ADDED_TRUST_STORE(result.subjectCommonName));
      setRefreshKey((k) => k + 1);
      handleBack();
    } catch (err: any) {
      // Check if it's a conflict (certificate already exists)
      if (err?.response?.status === 409 || err?.message?.includes('already exists')) {
        setError(SslCertificatesStrings.PAGE.CERTIFICATE_ALREADY_EXISTS_TRUST_STORE);
      }
      // Other errors are set by the API hook
    }
  }, [addCertificate, handleBack, setError, toast]);

  const handleDelete = useCallback(() => {
    if (!certificate) return;
    setDeleteDialogOpen(true);
  }, [certificate]);

  const handleDeleteConfirm = useCallback(async () => {
    if (!certificate) return;
    
    try {
      await deleteCertificate(certificate.id);
      toast.success(SslCertificatesStrings.PAGE.SSL_CERTIFICATE_REMOVED_TRUST_STORE(certificate.subjectCommonName));
      setRefreshKey((k) => k + 1);
      setDeleteDialogOpen(false);
      handleBack();
    } catch (err) {
      const message = err instanceof Error ? err.message : CommonStrings.MESSAGES.OPERATION_FAILED;
      toast.error(message);
      setDeleteDialogOpen(false);
    }
  }, [certificate, deleteCertificate, handleBack, toast]);

  // Render header based on view mode
  const renderHeader = () => {
    if (viewMode === 'list') {
      return (
        <PageHeader
          title={SslCertificatesStrings.PAGE.SSL_CERTIFICATES}
          description={SslCertificatesStrings.PAGE.MANAGE_TRUSTED_SSL_CERTIFICATES_USE}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#admin/settings') },
            { label: SslCertificatesStrings.PAGE.SSL_CERTIFICATES }
          ]}
          actions={canCreate && (
            <SettingsButton variant="primary" onClick={handleCreate} icon={Plus}>
              {SslCertificatesStrings.PAGE.ADD_CERTIFICATE}
            </SettingsButton>
          )}
        />
      );
    }

    const title = viewMode === 'create'
      ? SslCertificatesStrings.PAGE.ADD_SSL_CERTIFICATE
      : certificate
        ? SslCertificatesStrings.PAGE.CERTIFICATE(certificate.subjectCommonName)
        : SslCertificatesStrings.PAGE.CERTIFICATE_DETAILS;

    const lastBreadcrumb = viewMode === 'create'
      ? SslCertificatesStrings.PAGE.ADD
      : certificate?.subjectCommonName || CommonStrings.MESSAGES.LOADING;

    return (
      <PageHeader
        title={title}
        description={viewMode === 'create' ? SslCertificatesStrings.ADD_FORM.ADD_CERTIFICATE_TRUST_STORE_LOADING : undefined}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#admin/settings') },
          { label: SslCertificatesStrings.PAGE.SSL_CERTIFICATES, onClick: handleBack },
          { label: lastBreadcrumb }
        ]}
      />
    );
  };

  return (
    <Box
      className="ssl-certificates-page"
      data-testid="ssl-certificates-page"
      data-view={viewMode}
      data-loading={loading ? 'true' : 'false'}
      px={{ initial: '4', md: '6', lg: '6' }}
      py={{ initial: '4', md: '5', lg: '6' }}
    >
      <Box mb="4">
        {renderHeader()}
      </Box>

      {/* Alerts */}
      {error && (
        <Box className="ssl-certificates-page__alerts">
          <SettingsAlert type="error" onClose={() => setError(null)}>
            {error}
          </SettingsAlert>
        </Box>
      )}

      {/* Content */}
      <Box className="ssl-certificates-page__content">
        {viewMode === 'list' && (
          <SslCertificatesList
            key={refreshKey}
            onSelect={handleSelectCertificate}
            onCreate={handleCreate}
          />
        )}

        {viewMode === 'create' && (
          <SslCertificatesAddForm
            onSave={handleSave}
            onCancel={handleBack}
            loading={loading}
            onViewExisting={handleSelectCertificate}
          />
        )}

        {viewMode === 'detail' && (
          <SslCertificatesDetail
            certificate={certificate}
            loading={loading && !certificate}
            canDelete={canDelete}
            onDelete={handleDelete}
            onCancel={handleBack}
            
          />
        )}
      </Box>

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        open={deleteDialogOpen}
        testId="delete-certificate-dialog"
        onOpenChange={setDeleteDialogOpen}
        title={SslCertificatesStrings.PAGE.DELETE_CERTIFICATE_CONFIRM_TITLE}
        message={SslCertificatesStrings.PAGE.DELETE_CERTIFICATE_CONFIRM(certificate?.subjectCommonName)}
        confirmLabel={loading ? SslCertificatesStrings.PAGE.DELETING : SslCertificatesStrings.PAGE.DELETE_CERTIFICATE}
        variant="danger"
        onConfirm={handleDeleteConfirm}
        loading={loading}
      />
    </Box>
  );
}

export default SslCertificatesPage;

