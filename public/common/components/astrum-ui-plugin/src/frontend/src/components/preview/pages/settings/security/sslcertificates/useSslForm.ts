

import { useMemo, useCallback } from 'react';
import { useForm } from '../../../../../../interface/form';
import { useToast } from '../../../../shared';
import { createSslFormMachine, SslFormData } from './sslFormMachine';
import { CertificateSource, SslCertificate, CERTIFICATE_SOURCES } from './types';
import { useSslCertificatesApi } from './useSslCertificatesApi';

/**
 * Options for useSslForm hook
 */
export interface UseSslFormOptions {
  onSave?: () => void;
  onCancel: () => void;
}

/**
 * Extended context for reading certificate details from machine state
 */
interface SslExtendedContext {
  certificateDetails: SslCertificate | null;
}

/**
 * Custom hook for managing SSL certificate add form state.
 *
 * Uses XState form machine with source variant sub-states (remoteHost vs PEM).
 * Handles certificate loading from remote host or PEM parsing, and adding
 * to the trust store.
 */
export function useSslForm({ onSave, onCancel }: UseSslFormOptions) {
  const toast = useToast();
  const { addCertificate, loadCertificateDetails } = useSslCertificatesApi();

  // Create the form machine - stable across renders
  const machine = useMemo(() => createSslFormMachine(), []);

  // Use the form machine with save service override
  const form = useForm(machine, {
    actions: {
      onCancel,
    },
    services: {
      save: async (ctx: { data: SslFormData } & SslExtendedContext) => {
        try {
          // First load certificate details if not already loaded
          let certDetails = ctx.certificateDetails;
          if (!certDetails) {
            certDetails = await loadCertificateDetails(
              ctx.data.source,
              ctx.data.remoteHostUrl,
              ctx.data.pemContent
            );
          }

          // Get PEM content for adding to trust store
          const pemContent = ctx.data.source === CERTIFICATE_SOURCES.PEM
            ? ctx.data.pemContent
            : certDetails?.pem || '';

          if (!pemContent) {
            throw new Error('Certificate PEM content is required');
          }

          if (certDetails?.inTrustStore) {
            throw new Error('This certificate already exists in the trust store');
          }

          await addCertificate(pemContent);
          toast.success('Certificate added to trust store successfully');

          if (onSave) {
            onSave();
          }
        } catch (err) {
          toast.error(err instanceof Error ? err.message : 'Operation failed');
          throw err;
        }
      },
    },
  });

  // Access the raw state context
  const context = (form.state as { context: SslExtendedContext }).context;

  // Determine the current source variant from form data
  const currentSource = form.data.source as CertificateSource;

  // Handle source change via custom machine event
  const handleSourceChange = useCallback(
    (source: CertificateSource) => {
      form.send({ type: 'SOURCE_CHANGE', value: source } as any);
    },
    [form]
  );

  return {
    // Form state
    formData: form.data,
    errors: form.validationErrors,
    touched: form.touched,

    // Computed state
    isPristine: form.isPristine,
    isSaving: form.isSaving,
    hasValidationErrors: form.hasValidationErrors,
    isLoading: form.isLoading,
    saveError: form.saveError,

    // Certificate preview data
    certificateDetails: context.certificateDetails,

    // Current source variant
    currentSource,

    // Field helpers
    field: form.field,

    // Handlers
    handleChange: (field: string, value: unknown) =>
      form.send({ type: 'UPDATE', name: field, value }),
    handleBlur: (field: string) => form.send({ type: 'BLUR', name: field }),
    handleSourceChange,
    handleSubmit: form.submit,
    handleReset: form.reset,
    handleCancel: form.requestCancel,

    // Raw access
    state: form.state,
    send: form.send,
  };
}
