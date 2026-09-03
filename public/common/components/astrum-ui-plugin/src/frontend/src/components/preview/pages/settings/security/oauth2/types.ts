

/**
 * OAuth2 Configuration data
 */
export interface OAuth2Config {
  issuerUrl: string;
  redirectUri?: string;
  clientId: string;
  clientSecret: string;
  idpAuthorizationUrl: string;
  idpLogoutUrl: string;
  idpTokenUrl: string;
  idpJwksUrl: string;
  idpJwsAlgorithm: string;
  idpJwks?: string;
  usernameClaim: string;
  firstNameClaim: string;
  lastNameClaim: string;
  emailClaim: string;
  groupsClaim: string;
  exactMatchClaims?: string;
  authorizationCustomParams?: string;
  tokenRequestCustomParams?: string;
  /**
   * Whether to validate the IdP's certificate against the Astrum Repository
   * truststore. Must round-trip through load and save: the backend XO defaults
   * this to `false`, so omitting it from the payload silently disables a
   * setting the admin may have enabled elsewhere (ASTRUM-54266).
   */
  useTrustStore: boolean;
}

/**
 * Default OAuth2 configuration values
 */
export const DEFAULT_OAUTH2_CONFIG: OAuth2Config = {
  issuerUrl: '',
  clientId: '',
  clientSecret: '',
  idpAuthorizationUrl: '',
  idpLogoutUrl: '',
  idpTokenUrl: '',
  idpJwksUrl: '',
  idpJwsAlgorithm: '',
  idpJwks: '',
  usernameClaim: 'preferred_username',
  firstNameClaim: 'given_name',
  lastNameClaim: 'family_name',
  emailClaim: 'email',
  groupsClaim: '',
  exactMatchClaims: '',
  authorizationCustomParams: '',
  tokenRequestCustomParams: '',
  useTrustStore: false,
};

/**
 * Page props
 */
export interface OAuth2PageProps {
  className?: string;
}


