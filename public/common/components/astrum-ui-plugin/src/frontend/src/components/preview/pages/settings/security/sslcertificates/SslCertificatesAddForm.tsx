

import React, { useState, useCallback } from 'react';
import { Box, Text } from '@radix-ui/themes';

import {
  SettingsForm,
  SettingsFormSection,
  SettingsTextInput,
  SettingsTextArea,
  SettingsAlert,
  ConfirmDialog,
} from '../../../../shared/form';
import { useSslForm } from './useSslForm';
import { useSslCertificatesApi } from './useSslCertificatesApi';
import {
  SslCertificatesAddFormProps,
  SslCertificate,
  CERTIFICATE_SOURCES,
} from './types';
import { SslCertificatesDetail } from './SslCertificatesDetail';

import './SslCertificatesAddForm.scss';
import SslCertificatesStrings from '../../../../constants/pages/settings/security/SslCertificatesStrings';

/**
 * SslCertificatesAddForm - Form for adding SSL certificates
 * Uses XState form machine for field state, validation, and dirty tracking.
 * Keeps the two-step (load -> preview -> add) flow using local state.
 */
export function SslCertificatesAddForm({
  onSave,
  onCancel,
  loading: externalLoading = false,
  error: externalError,
  onViewExisting,
}: SslCertificatesAddFormProps) {
  const { loading: apiLoading, error: apiError, setError, loadCertificateDetails } = useSslCertificatesApi();

  // XState form hook for field state, validation, dirty tracking, source change
  const sslForm = useSslForm({ onSave: undefined, onCancel });

  // Local state for the two-step preview flow
  const [certificateDetails, setCertificateDetails] = useState<SslCertificate | null>(null);
  const [showPreview, setShowPreview] = useState(false);
  // Raised as soon as the loaded certificate turns out to be trusted already, so the user is
  // offered the existing certificate instead of a dead-end error (Classic UI parity).
  const [duplicateDialogOpen, setDuplicateDialogOpen] = useState(false);

  const loading = externalLoading || apiLoading || sslForm.isSaving;
  const error = externalError || apiError || sslForm.saveError || undefined;

  const handleLoadDetails = useCallback(async () => {
    // Touch all relevant fields to trigger validation display
    sslForm.handleBlur('remoteHostUrl');
    sslForm.handleBlur('pemContent');

    if (sslForm.hasValidationErrors) {
      return;
    }

    try {
      const details = await loadCertificateDetails(
        sslForm.formData.source,
        sslForm.formData.remoteHostUrl,
        sslForm.formData.pemContent
      );
      setCertificateDetails(details);
      setShowPreview(true);
      setDuplicateDialogOpen(Boolean(details?.inTrustStore));
      setError(null);
    } catch (_err) {
      // Error is set by the API hook
    }
  }, [sslForm, loadCertificateDetails, setError]);

  const handleSubmit = useCallback(async () => {
    if (!certificateDetails) {
      await handleLoadDetails();
      return;
    }

    // Already trusted: re-offer the existing certificate rather than attempting an add that
    // the backend would reject with a 409.
    if (certificateDetails.inTrustStore) {
      setDuplicateDialogOpen(true);
      setError(SslCertificatesStrings.ADD_FORM.CERTIFICATE_ALREADY_EXISTS_TRUST_STORE);
      return;
    }

    // Submit with PEM content
    const pemContent = sslForm.formData.source === CERTIFICATE_SOURCES.PEM
      ? sslForm.formData.pemContent
      : certificateDetails.pem || '';

    if (!pemContent) {
      setError(SslCertificatesStrings.ADD_FORM.CERTIFICATE_PEM_CONTENT_REQUIRED);
      return;
    }

    await onSave({
      ...sslForm.formData,
      pemContent,
    });
  }, [certificateDetails, sslForm.formData, onSave, handleLoadDetails, setError]);

  const handleViewExisting = useCallback(() => {
    if (certificateDetails?.id) {
      onViewExisting?.(certificateDetails.id);
    }
  }, [certificateDetails, onViewExisting]);

  const handleCancel = useCallback(() => {
    if (showPreview) {
      setShowPreview(false);
      setCertificateDetails(null);
      setDuplicateDialogOpen(false);
    } else {
      onCancel();
    }
  }, [showPreview, onCancel]);

  const handleSourceChange = useCallback((source: typeof CERTIFICATE_SOURCES.REMOTE_HOST | typeof CERTIFICATE_SOURCES.PEM) => {
    sslForm.handleSourceChange(source);
    setCertificateDetails(null);
    setShowPreview(false);
    setDuplicateDialogOpen(false);
    setError(null);
  }, [sslForm, setError]);

  const handleFieldChange = useCallback((field: string, value: string) => {
    sslForm.handleChange(field, value);
    setError(null);
    setCertificateDetails(null);
    setShowPreview(false);
    setDuplicateDialogOpen(false);
  }, [sslForm, setError]);

  return (
    <Box className="ssl-certificates-add-form">
      <SettingsForm
        testId="ssl-add-form"
        onSubmit={handleSubmit}
        onCancel={handleCancel}
        loading={loading}
        pristine={sslForm.isPristine}
        error={error}
        submitLabel={showPreview ? SslCertificatesStrings.ADD_FORM.ADD_CERTIFICATE : SslCertificatesStrings.ADD_FORM.LOAD_CERTIFICATE}
      >
        {!showPreview ? (
          <>
            {/* Source Selection */}
            <SettingsFormSection title={SslCertificatesStrings.ADD_FORM.CERTIFICATE_SOURCE} defaultOpen>
              <Box className="ssl-certificates-add-form__source-options">
                <Box className="ssl-certificates-add-form__radio-group">
                  <label className="ssl-certificates-add-form__radio-label">
                    <input
                      type="radio"
                      name="source"
                      value={CERTIFICATE_SOURCES.REMOTE_HOST}
                      checked={sslForm.formData.source === CERTIFICATE_SOURCES.REMOTE_HOST}
                      onChange={() => handleSourceChange(CERTIFICATE_SOURCES.REMOTE_HOST)}
                      className="ssl-certificates-add-form__radio-input"
                    />
                    <Box className="ssl-certificates-add-form__radio-content">
                      <Text size="2" weight="medium">{SslCertificatesStrings.ADD_FORM.LOAD_SERVER}</Text>
                      <Text size="1" className="ssl-certificates-add-form__radio-description">
                        {SslCertificatesStrings.ADD_FORM.ENTER_HOSTNAME_HOSTNAME_PORT_URL}
                      </Text>
                    </Box>
                  </label>

                  {sslForm.formData.source === CERTIFICATE_SOURCES.REMOTE_HOST && (
                    <Box className="ssl-certificates-add-form__field">
                      <SettingsTextInput
                        name="remoteHostUrl"
                        label={SslCertificatesStrings.ADD_FORM.HOSTNAME_URL}
                        value={sslForm.formData.remoteHostUrl}
                        onChange={(value: string) => handleFieldChange('remoteHostUrl', value)}
                        onBlur={() => sslForm.handleBlur('remoteHostUrl')}
                        error={sslForm.touched?.remoteHostUrl ? sslForm.errors?.remoteHostUrl : undefined}
                        placeholder={SslCertificatesStrings.ADD_FORM.EXAMPLE_COM_EXAMPLE_COM_443_PLACEHOLDER}
                        helpText={SslCertificatesStrings.ADD_FORM.REMOTE_SERVER_HOSTNAME_RETRIEVE_CERTIFICATE}
                        required
                      />
                    </Box>
                  )}
                </Box>

                <Box className="ssl-certificates-add-form__radio-group">
                  <label className="ssl-certificates-add-form__radio-label">
                    <input
                      type="radio"
                      name="source"
                      value={CERTIFICATE_SOURCES.PEM}
                      checked={sslForm.formData.source === CERTIFICATE_SOURCES.PEM}
                      onChange={() => handleSourceChange(CERTIFICATE_SOURCES.PEM)}
                      className="ssl-certificates-add-form__radio-input"
                    />
                    <Box className="ssl-certificates-add-form__radio-content">
                      <Text size="2" weight="medium">{SslCertificatesStrings.ADD_FORM.PASTE_PEM}</Text>
                      <Text size="1" className="ssl-certificates-add-form__radio-description">
                        {SslCertificatesStrings.ADD_FORM.PASTE_CERTIFICATE_CONTENT_PEM_FORMAT}
                      </Text>
                    </Box>
                  </label>

                  {sslForm.formData.source === CERTIFICATE_SOURCES.PEM && (
                    <Box className="ssl-certificates-add-form__field">
                      <SettingsTextArea
                        name="pemContent"
                        label={SslCertificatesStrings.ADD_FORM.PASTE_CERTIFICATE_PEM}
                        value={sslForm.formData.pemContent}
                        onChange={(value: string) => handleFieldChange('pemContent', value)}
                        onBlur={() => sslForm.handleBlur('pemContent')}
                        error={sslForm.touched?.pemContent ? sslForm.errors?.pemContent : undefined}
                        placeholder="-----BEGIN CERTIFICATE-----&#10;...&#10;-----END CERTIFICATE-----"
                        helpText={SslCertificatesStrings.ADD_FORM.PASTE_PEM_ENCODED_CERTIFICATE_CONTENT}
                        rows={8}
                        required
                      />
                    </Box>
                  )}
                </Box>
              </Box>
            </SettingsFormSection>
          </>
        ) : (
          <>
            {/* Certificate Preview */}
            {certificateDetails && (
              <>
                {certificateDetails.inTrustStore && (
                  <Box className="ssl-certificates-add-form__alerts">
                    <SettingsAlert type="error">
                      {SslCertificatesStrings.ADD_FORM.CERTIFICATE_ALREADY_EXISTS_TRUST_STORE}
                    </SettingsAlert>
                  </Box>
                )}
                {/* The embedded detail keeps its own action bar, so its cancel has to step back
                    through this form's two-step flow — preview -> form, then form -> list. Wiring it
                    to a no-op rendered a live but inert button (SettingsForm only omits the button
                    when onCancel is absent entirely). onDelete stays a no-op: canDelete is false,
                    so no delete button is rendered to reach it. */}
                <SslCertificatesDetail
                  certificate={certificateDetails}
                  loading={false}
                  canDelete={false}
                  onDelete={() => {}}
                  onCancel={handleCancel}
                  showTrustWarning
                />
              </>
            )}
          </>
        )}
      </SettingsForm>

      {/* Already-trusted certificate: offer the existing one instead of a dead-end error */}
      <ConfirmDialog
        open={duplicateDialogOpen}
        testId="certificate-already-exists-dialog"
        onOpenChange={setDuplicateDialogOpen}
        title={SslCertificatesStrings.ADD_FORM.CERTIFICATE_ALREADY_EXISTS}
        message={SslCertificatesStrings.ADD_FORM.CERTIFICATE_ALREADY_EXISTS_CANNOT_ADDED}
        confirmLabel={SslCertificatesStrings.ADD_FORM.VIEW_CERTIFICATE}
        variant="warning"
        onConfirm={handleViewExisting}
      />
    </Box>
  );
}

export default SslCertificatesAddForm;
