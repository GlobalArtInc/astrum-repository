

/**
 * HTTP settings configuration data model
 */
export interface HttpConfiguration {
  userAgentSuffix: string;
  timeout: number | null;
  retries: number | null;
  httpEnabled: boolean;
  httpHost: string;
  httpPort: number | null;
  /** '' = no auth, 'username' = username/password (+ optional NTLM) */
  httpAuthType: string;
  httpAuthUsername: string;
  httpAuthPassword: string;
  httpAuthNtlmHost: string;
  httpAuthNtlmDomain: string;
  httpsEnabled: boolean;
  httpsHost: string;
  httpsPort: number | null;
  /** '' = no auth, 'username' = username/password (+ optional NTLM) */
  httpsAuthType: string;
  httpsAuthUsername: string;
  httpsAuthPassword: string;
  httpsAuthNtlmHost: string;
  httpsAuthNtlmDomain: string;
  nonProxyHosts: string[];
}

/**
 * Props for HttpPage component
 */
export interface HttpPageProps {
  className?: string;
}

/**
 * Default HTTP configuration
 */
export const DEFAULT_HTTP_CONFIGURATION: HttpConfiguration = {
  userAgentSuffix: '',
  timeout: null,
  retries: null,
  httpEnabled: false,
  httpHost: '',
  httpPort: null,
  httpAuthType: '',
  httpAuthUsername: '',
  httpAuthPassword: '',
  httpAuthNtlmHost: '',
  httpAuthNtlmDomain: '',
  httpsEnabled: false,
  httpsHost: '',
  httpsPort: null,
  httpsAuthType: '',
  httpsAuthUsername: '',
  httpsAuthPassword: '',
  httpsAuthNtlmHost: '',
  httpsAuthNtlmDomain: '',
  nonProxyHosts: [],
};

/**
 * Validation errors for HTTP configuration form
 */
export interface HttpValidationErrors {
  timeout?: string;
  retries?: string;
  httpHost?: string;
  httpPort?: string;
  httpAuthUsername?: string;
  httpsHost?: string;
  httpsPort?: string;
  httpsAuthUsername?: string;
}


