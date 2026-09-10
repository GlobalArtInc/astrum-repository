

/**
 * Email server configuration data model
 */
export interface EmailConfiguration {
  enabled: boolean;
  host: string;
  port: number;
  // UI-only derived field: set to `username !== ''` on load, not persisted in the REST API.
  // On save, emailConfigToRest gates username/password on this flag but does not send it.
  useAuthentication: boolean;
  username: string;
  password: string;
  fromAddress: string;
  subjectPrefix: string;
  startTlsEnabled: boolean;
  startTlsRequired: boolean;
  sslOnConnectEnabled: boolean;
  sslCheckServerIdentityEnabled: boolean;
  astrumTrustStoreEnabled: boolean;
}

/**
 * Email verification result
 */
export interface EmailVerificationResult {
  success: boolean;
  reason?: string;
}

/**
 * Props for EmailPage component
 */
export interface EmailPageProps {
  className?: string;
}

/**
 * Default email configuration
 */
export const DEFAULT_EMAIL_CONFIGURATION: EmailConfiguration = {
  enabled: false,
  host: '',
  port: 25,
  useAuthentication: false,
  username: '',
  password: '',
  fromAddress: '',
  subjectPrefix: '',
  startTlsEnabled: false,
  startTlsRequired: false,
  sslOnConnectEnabled: false,
  sslCheckServerIdentityEnabled: false,
  astrumTrustStoreEnabled: false,
};

/**
 * Validation errors for email configuration form
 */
export interface EmailValidationErrors {
  host?: string;
  port?: string;
  fromAddress?: string;
}
