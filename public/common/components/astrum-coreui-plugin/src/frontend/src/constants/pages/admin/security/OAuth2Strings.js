
import { faIdCard } from '@fortawesome/free-solid-svg-icons';

export default {
  OAUTH2_CONFIGURATION: {
    MENU: {
      text: 'OpenID Connect',
      description: 'Sign in with any OpenID Connect provider',
      icon: faIdCard
    },
    FIELDS: {
      ISSUER_URL: 'Issuer URL',
      REDIRECT_URI: 'Redirect URI',
      IDP_JWKS_URL: 'JSON Web Key URL',
      IDP_JWS_ALGORITHM: 'JWT Signature Algorithm',
      IDP_JWKS: 'JSON Web Keys',
      USERNAME_CLAIM: 'Username Claim',
      FIRST_NAME_CLAIM: 'First Name Claim',
      LAST_NAME_CLAIM: 'Last Name Claim',
      EMAIL_CLAIM: 'Email Claim',
      GROUPS_CLAIM: 'Groups Claim',
      EXACT_MATCH_CLAIMS: 'Exact Match Claims',
      CLIENT_ID: 'Client ID',
      CLIENT_SECRET: 'Client Secret',
      IDP_AUTHORIZATION_URL: 'Authorization URL',
      IDP_LOGOUT_URL: 'Logout URL',
      IDP_TOKEN_URL: 'Token URL',
      ADVANCED_SETTINGS: 'Advanced Settings',
      AUTHORIZATION_CUSTOM_PARAMS: 'Authorization Custom Parameters',
      TOKEN_REQUEST_CUSTOM_PARAMS: 'Token Request Custom Parameters',
      OPTIONAL_JWKS: 'Optional: Provide either JSON Web Keys or a URL to fetch the keys from the Identity Provider'
    },
    MESSAGES: {
      LOAD_ERROR: 'An error occurred while attempting to load the OAuth 2.0 configuration',
      SAVE_ERROR: 'An error occurred while attempting to save the OAuth 2.0 configuration',
      SAVE_SUCCESS: 'OAuth 2.0 configuration updated'
    },
    LABELS: {
      FIELDS: 'OAuth 2.0 Field Mappings',
      OAUTH_2_0_CONFIGURATION: "OAuth 2.0 Configuration",
      OIDC_SETTINGS: 'OIDC Settings',
      JWT_SETTINGS: 'JWT Settings',
      CLAIM_MAPPINGS: 'Claim Mappings',
      SUBLABELS: {
        REQUIRED_FIELDS: 'Required fields are marked with an asterisk',
        ISSUER_URL: "The issuer of your identity provider, for example https://sso.example.com. Endpoints and keys are read from its .well-known/openid-configuration; the fields below override them when set.",
        REDIRECT_URI: 'Register this redirect URI for the client in your identity provider.',
        CLIENT_ID: 'The client ID registered with your identity provider.',
        CLIENT_SECRET: 'The client secret registered with your identity provider. Leave empty for a public client that uses PKCE only.',
        IDP_AUTHORIZATION_URL: "Overrides the authorization endpoint from the issuer's discovery document.",
        IDP_LOGOUT_URL: "Overrides the end-session endpoint from the issuer's discovery document.",
        IDP_TOKEN_URL: "Overrides the token endpoint from the issuer's discovery document.",
        IDP_JWKS_URL: "Overrides the JWKS URI from the issuer's discovery document.",
        USERNAME_CLAIM: "The claim that becomes the Astrum user ID. Defaults to preferred_username, then sub.",
        FIRST_NAME_CLAIM: "The claim with the user's first name. Defaults to given_name, then name.",
        LAST_NAME_CLAIM: "The claim with the user's last name. Defaults to family_name.",
        EMAIL_CLAIM: "The claim with the user's email address. Defaults to email.",
        GROUPS_CLAIM: "The claim with the user's groups, for example groups or realm_access.roles. Groups named like Astrum roles grant those roles.",
        IDP_JWS_ALGORITHM: "Restricts ID tokens to one signing algorithm (RS256, RS384, RS512, ES256, ES384 or ES512). Leave empty to accept any of them.",
        IDP_JWKS: "Paste the JWKS JSON here if not using the JWKS URI.",
        AUTHORIZATION_CUSTOM_PARAMS: "Custom parameters to include in the authorization request (JSON format).",
        TOKEN_REQUEST_CUSTOM_PARAMS: "Custom parameters to include in the token request (JSON format).",
        EXACT_MATCH_CLAIMS: "Claims that must match exactly (JSON format)."
      }
    }
  }
};