

import { useState, useCallback } from 'react';
import { ExtAPIUtils } from '../../../../../../interface/ExtAPIUtils';
import { APIConstants } from '../../../../../../constants/APIConstants';
import { SslCertificate, CertificateSource, CERTIFICATE_SOURCES } from './types';
import SslCertificatesStrings from '../../../../constants/pages/settings/security/SslCertificatesStrings';

const { EXT } = APIConstants;

// The trust store is managed over ExtDirect (ssl_TrustStore.read / create / remove), as in Classic UI: the server
// has no REST v1 resource for it, so /v1/security/ssl/truststore answers 404.
//
// Inspecting a certificate BEFORE adding it goes over ExtDirect too (ssl_Certificate.details / .retrieveFromHost).
// Those are read-only (@RequiresPermissions astrum:ssl-truststore:read) and return `inTrustStore`, which drives
// duplicate detection in the add form (ASTRUM-54265); ssl_TrustStore.create imports the certificate, so it must never
// be used to preview one.

/**
 * Parse remote host URL and extract hostname, port, and protocol
 */
function parseRemoteHostUrl(url: string): [string, number | null, string | null] {
  const hasProtocol = url.startsWith('http://') || url.startsWith('https://');
  const urlStr = hasProtocol ? url : `https://${url}`;
  
  try {
    const { protocol, hostname, port } = new URL(urlStr);
    const portNumber = port ? parseInt(port, 10) : null;
    const protocolHint = hasProtocol ? protocol : null;
    return [hostname, portNumber, protocolHint];
  } catch (_error) {
    // If URL parsing fails, try to extract hostname manually
    const hostname = url.replace(/^https?:\/\//, '').split(':')[0].split('/')[0];
    return [hostname, null, null];
  }
}

/**
 * The message to show for a failed request: the server's own message, else the given text, since the generic
 * transport message of the HTTP client is neither helpful nor translated.
 */
function messageOf(err: any, fallback: string): string {
  const serverMessage = err?.response?.data?.message;
  if (serverMessage) {
    return serverMessage;
  }
  if (err?.response || err?.isAxiosError) {
    return fallback;
  }
  return err?.message || fallback;
}

/**
 * Custom hook for SSL Certificates API operations
 */
export function useSslCertificatesApi() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetch all certificates
   */
  const fetchCertificates = useCallback(async (): Promise<SslCertificate[]> => {
    try {
      const response = await ExtAPIUtils.extAPIRequest(EXT.SSL_TRUST_STORE.ACTION, EXT.SSL_TRUST_STORE.METHODS.READ);
      ExtAPIUtils.checkForError(response);
      const certificates = ExtAPIUtils.extractResult(response);
      return Array.isArray(certificates) ? certificates : [];
    } catch (err: any) {
      console.error('Failed to fetch certificates:', err);
      throw new Error(messageOf(err, SslCertificatesStrings.API.FAILED_LOAD_CERTIFICATES));
    }
  }, []);

  /**
   * Retrieve (without importing) the certificate presented by a remote host.
   * Read-only: ssl_Certificate.retrieveFromHost.
   */
  const fetchCertificateFromHost = useCallback(async (
    host: string,
    port: number | null = null,
    protocolHint: string | null = null
  ): Promise<SslCertificate> => {
    try {
      const response = await ExtAPIUtils.extAPIRequest(EXT.SSL.ACTION, EXT.SSL.METHODS.RETRIEVE_FROM_HOST, {
        data: [host, port, protocolHint],
      });
      ExtAPIUtils.checkForError(response);
      return ExtAPIUtils.extractResult(response);
    } catch (err: any) {
      console.error('Failed to fetch certificate from host:', err);
      throw new Error(messageOf(err, SslCertificatesStrings.API.FAILED_RETRIEVE_CERTIFICATE_HOST));
    }
  }, []);

  /**
   * Parse a PEM-encoded certificate so it can be previewed before being trusted.
   *
   * Read-only: ssl_Certificate.details. This MUST NOT go through
   * POST /v1/security/ssl/truststore — that endpoint imports the certificate, so using
   * it to "preview" silently added the certificate and made the subsequent add fail with
   * a 409, reporting failure for a certificate that had in fact been trusted
   * (ASTRUM-54265).
   */
  const getCertificateDetails = useCallback(async (pemContent: string): Promise<SslCertificate> => {
    try {
      const response = await ExtAPIUtils.extAPIRequest(EXT.SSL.ACTION, EXT.SSL.METHODS.DETAILS, {
        data: [pemContent],
      });
      ExtAPIUtils.checkForError(response);
      return ExtAPIUtils.extractResult(response);
    } catch (err: any) {
      console.error('Failed to get certificate details:', err);
      throw new Error(messageOf(err, SslCertificatesStrings.API.FAILED_PARSE_CERTIFICATE));
    }
  }, []);

  /**
   * Add certificate to truststore
   */
  const addCertificate = useCallback(async (pemContent: string): Promise<SslCertificate> => {
    setLoading(true);
    setError(null);
    try {
      const response = await ExtAPIUtils.extAPIRequest(EXT.SSL_TRUST_STORE.ACTION, EXT.SSL_TRUST_STORE.METHODS.CREATE, {
        data: [pemContent],
      });
      ExtAPIUtils.checkForError(response);
      return ExtAPIUtils.extractResult(response);
    } catch (err: any) {
      const message = messageOf(err, SslCertificatesStrings.API.FAILED_ADD_CERTIFICATE);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Delete certificate from truststore
   */
  const deleteCertificate = useCallback(async (certificateId: string): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      const response = await ExtAPIUtils.extAPIRequest(EXT.SSL_TRUST_STORE.ACTION, EXT.SSL_TRUST_STORE.METHODS.REMOVE, {
        data: [certificateId],
      });
      ExtAPIUtils.checkForError(response);
    } catch (err: any) {
      const message = messageOf(err, SslCertificatesStrings.API.FAILED_DELETE_CERTIFICATE);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Load certificate details based on source type
   */
  const loadCertificateDetails = useCallback(async (
    source: CertificateSource,
    remoteHostUrl: string,
    pemContent: string
  ): Promise<SslCertificate> => {
    setLoading(true);
    setError(null);
    try {
      if (source === CERTIFICATE_SOURCES.REMOTE_HOST && remoteHostUrl) {
        const [hostname, port, protocolHint] = parseRemoteHostUrl(remoteHostUrl);
        return await fetchCertificateFromHost(hostname, port, protocolHint);
      } else if (source === CERTIFICATE_SOURCES.PEM && pemContent) {
        return await getCertificateDetails(pemContent);
      } else {
        throw new Error(SslCertificatesStrings.API.INVALID_SOURCE_MISSING_DATA);
      }
    } catch (err: any) {
      const message = err instanceof Error ? err.message : SslCertificatesStrings.API.FAILED_LOAD_CERTIFICATE_DETAILS;
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, [fetchCertificateFromHost, getCertificateDetails]);

  return {
    loading,
    error,
    setError,
    fetchCertificates,
    fetchCertificateFromHost,
    getCertificateDetails,
    addCertificate,
    deleteCertificate,
    loadCertificateDetails,
  };
}

export default useSslCertificatesApi;
