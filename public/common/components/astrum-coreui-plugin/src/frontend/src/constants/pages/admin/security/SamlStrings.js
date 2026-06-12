
import { faIdCard } from '@fortawesome/free-solid-svg-icons';

export default {
  SAML_CONFIGURATION: {
    MENU: {
      text: 'SAML',
      description: 'SAML Identity Provider Configuration',
      icon: faIdCard
    },
    FIELDS: {
      idpMetadataLabel: 'SAML Identity Provider Metadata XML',

      entityIdUriLabel: 'Entity ID URI',
      entityIdUriDescription: 'Unique identifier for this Service Provider.  Normally the default Entity ID can be used unless Astrum does not have a fixed URL and or port.',
      entityIdUriValidationError: 'Entity ID must be a URI',

      validateResponseSignatureLabel: 'Validate Response Signature',
      validateResponseSignatureDescription: 'By default, if a signing key is found in the IdP metadata, then Astrum will attempt to validate signatures on the response.',

      validateAssertionSignatureLabel: 'Validate Assertion Signature',
      validateAssertionSignatureDescription: 'By default, if a signing key is found in the IdP metadata, then Astrum will attempt to validate signatures on the assertions.',

      defaultRoleLabel: 'Default Role Mapping',
      defaultRoleDescription: 'The role a user should be assigned if no other roles were able to be mapped to the user',

      usernameAttrLabel: 'Username',

      firstNameAttrLabel: 'First Name',

      lastNameAttrLabel: 'Last Name',

      emailAttrLabel: 'Email',

      roleAttrLabel: 'Roles/Groups',
      roleFieldDescription: 'Map Identity Provider roles or groups to Astrum Repository Manager roles'
    },
    MESSAGES: {
      LOAD_ERROR: 'An error occurred while attempting to load the SAML Identity Provider configuration',
      SAVE_ERROR: 'An error occurred while attempting to save the SAML Identity Provider configuration',
      SAVE_SUCCESS: 'SAML Identity Provider configuration updated'
    },
    LABELS: {
      FIELDS: 'IDP Field Mappings'
    }
  }
};