import { pageLanguage } from '../../../../../../interface/plural';


/**
 * SSL Certificate data model matching the backend API
 */
export interface SslCertificate {
  id: string;
  subjectCommonName: string;
  subjectOrganization: string;
  subjectOrganizationalUnit?: string;
  issuerCommonName: string;
  issuerOrganization: string;
  issuerOrganizationalUnit?: string;
  fingerprint: string;
  issuedOn?: number; // Timestamp
  expiresOn?: number; // Timestamp
  inTrustStore?: boolean;
  pem?: string; // PEM content (for adding)
}

/**
 * Certificate source type for adding certificates
 */
export type CertificateSource = 'remoteHost' | 'PEM';

/**
 * Add certificate form data
 */
export interface AddCertificateFormData {
  source: CertificateSource;
  remoteHostUrl: string;
  pemContent: string;
}

/**
 * Form validation errors
 */
export interface CertificateFormErrors {
  remoteHostUrl?: string;
  pemContent?: string;
}

/**
 * Sort direction type
 */
export type SortDirection = 'asc' | 'desc' | null;

/**
 * Sortable fields for certificates list
 */
export type CertificateSortField = 'subjectCommonName' | 'subjectOrganization' | 'issuerOrganization' | 'fingerprint';

/**
 * Props for SslCertificatesPage component
 */
export interface SslCertificatesPageProps {
  className?: string;
}

/**
 * Props for SslCertificatesList component
 */
export interface SslCertificatesListProps {
  onSelect: (certificateId: string) => void;
  onCreate: () => void;
}

/**
 * Props for SslCertificatesDetail component
 */
export interface SslCertificatesDetailProps {
  certificate: SslCertificate | null;
  loading: boolean;
  canDelete: boolean;
  onDelete: () => void;
  onCancel: () => void;
  error?: string;
  /**
   * Shows the "trust the issuing CA" warning. Only meaningful while previewing a certificate that
   * has not been added yet — on an already-trusted certificate the advice is about a decision the
   * user has already made (ASTRUM-54265).
   */
  showTrustWarning?: boolean;
}

/**
 * Props for SslCertificatesAddForm component
 */
export interface SslCertificatesAddFormProps {
  onSave: (data: AddCertificateFormData) => Promise<void>;
  onCancel: () => void;
  loading?: boolean;
  error?: string;
  /**
   * Navigate to an already-trusted certificate. Used when the loaded certificate turns out
   * to be in the trust store already, so the user has a way to reach it instead of a
   * dead-end error (Classic UI's SslCertificatesAlreadyExistsModal).
   */
  onViewExisting?: (certificateId: string) => void;
}

/**
 * Certificate source options
 */
export const CERTIFICATE_SOURCES = {
  REMOTE_HOST: 'remoteHost' as const,
  PEM: 'PEM' as const,
} as const;

/**
 * Format timestamp to readable date string
 */
export const formatDate = (timestamp?: number): string => {
  if (!timestamp) return '';
  return new Date(timestamp).toLocaleString(pageLanguage());
};

/**
 * Check if certificate is expired or expiring soon
 */
export const isCertificateExpiring = (certificate: SslCertificate, daysThreshold: number = 30): boolean => {
  if (!certificate.expiresOn) return false;
  const now = Date.now();
  const threshold = daysThreshold * 24 * 60 * 60 * 1000;
  return certificate.expiresOn - now < threshold;
};

/**
 * Check if certificate is expired
 */
export const isCertificateExpired = (certificate: SslCertificate): boolean => {
  if (!certificate.expiresOn) return false;
  return certificate.expiresOn < Date.now();
};
