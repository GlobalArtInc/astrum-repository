

import { assign } from 'xstate';
import { createFormMachine, type FormContext, type ValidationErrors } from '../../../../../../interface/form';
import ValidationUtils from '../../../../../../interface/ValidationUtils';

import {
  LdapServer,
  LdapFormData,
  LdapSchemaTemplate,
  DEFAULT_LDAP_SERVER,
} from './types';
import CommonStrings from '../../../../constants/CommonStrings';
import LdapStrings from '../../../../constants/pages/settings/security/LdapStrings';

/**
 * Validate LDAP connection fields.
 * Checks name, host, port, search base DN, auth credentials, and the three
 * numeric connection-settings fields (connectionTimeout, connectionRetryDelay,
 * maxIncidentsCount). All three are @NotNull on the backend, so a cleared
 * (undefined) value is an error — "This field is required" — not a pass.
 *
 * @param isCreate When true (the default), a password is required for
 *   authenticated connections. In edit mode (isCreate=false) the bind password
 *   is NOT required: the REST API never returns stored passwords, so a blank
 *   field means "keep the existing password". This mirrors the
 *   `required={isCreate}` behavior in LdapForm.tsx so the machine's continuous
 *   validation does not flag a genuinely-optional field as invalid on edit.
 * @param existingNames Names of other LDAP servers, used to reject a name
 *   collision in both create and edit mode. The caller is responsible for
 *   excluding the server's own current name (e.g. when editing) so that
 *   saving a server without renaming it does not flag itself as a duplicate.
 */
export function validateConnection(data: LdapFormData, isCreate = true, existingNames: string[] = []): ValidationErrors {
  const errors: ValidationErrors = {};

  if (!data.name?.trim()) {
    errors.name = CommonStrings.MESSAGES.NAME_REQUIRED;
  } else if (existingNames.includes(data.name.trim())) {
    errors.name = LdapStrings.VALIDATION.NAME_ALREADY_USE;
  }

  if (!(data.protocol && ['ldap', 'ldaps'].includes(data.protocol))) {
    errors.protocol = LdapStrings.VALIDATION.PROTOCOL_REQUIRED;
  }

  if (!data.host?.trim()) {
    errors.host = LdapStrings.VALIDATION.HOSTNAME_REQUIRED;
  } else if (!ValidationUtils.isHost(data.host)) {
    errors.host = LdapStrings.VALIDATION.NOT_VALID_HOSTNAME_IP_ADDRESS;
  }

  if (!data.port || data.port < 1 || data.port > 65535) {
    errors.port = LdapStrings.VALIDATION.PORT_MUST_BETWEEN_1_65535;
  }

  if (!data.searchBase?.trim()) {
    errors.searchBase = LdapStrings.VALIDATION.SEARCH_BASE_DN_REQUIRED;
  }

  // Auth credentials required when scheme is not anonymous
  if (data.authScheme !== 'none') {
    if (!data.authUsername?.trim()) {
      errors.authUsername = LdapStrings.VALIDATION.USERNAME_REQUIRED_AUTHENTICATED_CONNECTION;
    }
    if (isCreate && !data.authPassword) {
      errors.authPassword = LdapStrings.VALIDATION.PASSWORD_REQUIRED_AUTHENTICATED_CONNECTION;
    }
  }

  // Numeric connection rules; all three fields are @NotNull on the backend
  // (LdapServerXo / LdapServerConnectionXO). Required check runs first so
  // that a cleared field shows "This field is required" rather than silently
  // passing validation. NaN, below-range, and above-range all show the same
  // range message so users get actionable feedback regardless of how the value
  // is invalid (ASTRUM-54076).
  if (data.connectionTimeout == null) {
    errors.connectionTimeout = LdapStrings.VALIDATION.FIELD_REQUIRED;
  } else if (Number.isNaN(data.connectionTimeout) || !Number.isInteger(data.connectionTimeout) || data.connectionTimeout < 1 || data.connectionTimeout > 3600) {
    errors.connectionTimeout = LdapStrings.VALIDATION.MUST_NUMBER_BETWEEN_1_3600;
  }

  if (data.connectionRetryDelay == null) {
    errors.connectionRetryDelay = LdapStrings.VALIDATION.FIELD_REQUIRED;
  } else if (Number.isNaN(data.connectionRetryDelay) || !Number.isInteger(data.connectionRetryDelay) || data.connectionRetryDelay < 0) {
    errors.connectionRetryDelay = LdapStrings.VALIDATION.MUST_NUMBER_0_GREATER;
  }

  if (data.maxIncidentsCount == null) {
    errors.maxIncidentsCount = LdapStrings.VALIDATION.FIELD_REQUIRED;
  } else if (Number.isNaN(data.maxIncidentsCount) || !Number.isInteger(data.maxIncidentsCount) || data.maxIncidentsCount < 0) {
    errors.maxIncidentsCount = LdapStrings.VALIDATION.MUST_NUMBER_0_GREATER;
  }

  return errors;
}

/**
 * Validate LDAP user and group mapping fields.
 * Checks user object class, ID/name/email attributes, and group fields
 * when LDAP groups as roles is enabled.
 */
export function validateUserGroup(data: LdapFormData): ValidationErrors {
  const errors: ValidationErrors = {};

  if (!data.userObjectClass?.trim()) {
    errors.userObjectClass = LdapStrings.VALIDATION.USER_OBJECT_CLASS_REQUIRED;
  }

  if (!data.userIdAttribute?.trim()) {
    errors.userIdAttribute = LdapStrings.VALIDATION.USER_ID_ATTRIBUTE_REQUIRED;
  }

  if (!data.userRealNameAttribute?.trim()) {
    errors.userRealNameAttribute = LdapStrings.VALIDATION.REAL_NAME_ATTRIBUTE_REQUIRED;
  }

  if (!data.userEmailAddressAttribute?.trim()) {
    errors.userEmailAddressAttribute = LdapStrings.VALIDATION.EMAIL_ATTRIBUTE_REQUIRED;
  }

  if (data.ldapGroupsAsRoles) {
    if (data.groupType === 'static') {
      if (!data.groupObjectClass?.trim()) {
        errors.groupObjectClass = LdapStrings.VALIDATION.GROUP_OBJECT_CLASS_REQUIRED;
      }
      if (!data.groupIdAttribute?.trim()) {
        errors.groupIdAttribute = LdapStrings.VALIDATION.GROUP_ID_ATTRIBUTE_REQUIRED;
      }
      if (!data.groupMemberAttribute?.trim()) {
        errors.groupMemberAttribute = LdapStrings.VALIDATION.GROUP_MEMBER_ATTRIBUTE_REQUIRED;
      }
      if (!data.groupMemberFormat?.trim()) {
        errors.groupMemberFormat = LdapStrings.VALIDATION.GROUP_MEMBER_FORMAT_REQUIRED;
      }
    } else if (data.groupType === 'dynamic') {
      if (!data.userMemberOfAttribute?.trim()) {
        errors.userMemberOfAttribute = LdapStrings.VALIDATION.USER_MEMBER_ATTRIBUTE_REQUIRED;
      }
    }
  }

  return errors;
}

/**
 * Validate all LDAP form fields (connection + user/group mapping).
 * `isCreate` controls whether the bind password is required (create) or optional (edit).
 * `existingNames` is used in both modes to reject a name that already belongs to another
 * server; the caller excludes the server's own current name in edit mode.
 */
export function validateLdap(data: LdapFormData, isCreate = true, existingNames: string[] = []): ValidationErrors {
  return {
    ...validateConnection(data, isCreate, existingNames),
    ...validateUserGroup(data),
  };
}

/**
 * Map a loaded LDAP server (edit mode) to the flat form data shape, or fall back to defaults
 * (create mode). Single source of truth used both for the machine's initial context and its
 * `load` service result, so the two can never drift apart when a field is added.
 */
function serverToFormData(server?: LdapServer | null): LdapFormData {
  return server
    ? {
        id: server.id,
        name: server.name,
        protocol: server.protocol,
        useTrustStore: server.useTrustStore,
        host: server.host,
        port: server.port,
        searchBase: server.searchBase,
        authScheme: server.authScheme,
        authRealm: server.authRealm,
        authUsername: server.authUsername,
        authPassword: server.authPassword,
        connectionTimeout: server.connectionTimeout,
        connectionRetryDelay: server.connectionRetryDelay,
        maxIncidentsCount: server.maxIncidentsCount,
        userBaseDn: server.userBaseDn,
        userSubtree: server.userSubtree,
        userObjectClass: server.userObjectClass,
        userLdapFilter: server.userLdapFilter,
        userIdAttribute: server.userIdAttribute,
        userRealNameAttribute: server.userRealNameAttribute,
        userEmailAddressAttribute: server.userEmailAddressAttribute,
        userPasswordAttribute: server.userPasswordAttribute,
        ldapGroupsAsRoles: server.ldapGroupsAsRoles,
        groupType: server.groupType,
        groupBaseDn: server.groupBaseDn,
        groupSubtree: server.groupSubtree,
        groupObjectClass: server.groupObjectClass,
        groupIdAttribute: server.groupIdAttribute,
        groupMemberAttribute: server.groupMemberAttribute,
        groupMemberFormat: server.groupMemberFormat,
        userMemberOfAttribute: server.userMemberOfAttribute,
      }
    : { ...DEFAULT_LDAP_SERVER };
}

/**
 * Create an LDAP form machine with XState.
 *
 * The LDAP form is a flat form (no type variants / editingConfig) that covers:
 * - Connection settings (name, protocol, host, port, search base, auth)
 * - User mapping (object class, ID/name/email attributes, filters)
 * - Group mapping (conditional on ldapGroupsAsRoles flag)
 *
 * The load service fetches the LDAP server data (if editing) and
 * schema templates for pre-filling common configurations.
 */
export function createLdapFormMachine(
  serverId: string | undefined,
  preloadedServer?: LdapServer | null,
  existingNamesRef?: { current: string[] }
) {
  // Edit mode (an existing server was preloaded, or is being loaded by ID)
  // does not require a password to be re-entered on every save; create mode
  // does. Must match useLdapForm.ts's `!(serverId || server)` exactly - that
  // hook's isCreate is what LdapForm.tsx and its callers see, so a diverging
  // formula here (e.g. checking preloadedServer alone, ignoring serverId)
  // would let this security-relevant gate silently disagree with the rest of
  // the form for a serverId-without-preloaded-server caller.
  const isCreate = !(serverId || preloadedServer);

  // Initialize form data from preloaded server if available, otherwise use defaults.
  // This ensures the form displays correct values immediately before load service completes.
  // Shared helper (serverToFormData) is the single source of truth used both here and by the
  // load service, so the two can never drift apart when a field is added.
  const initialFormData: LdapFormData = serverToFormData(preloadedServer);

  return createFormMachine({
    id: `ldap-form-${serverId ?? 'new'}`,
    context: {
      data: initialFormData,
      // Reference data populated by load service
      server: preloadedServer ?? (null as LdapServer | null),
      templates: [] as LdapSchemaTemplate[],
    },
    actions: {
      validate: assign((ctx: FormContext<LdapFormData>) => ({
        validationErrors: validateLdap(ctx.data, isCreate, existingNamesRef?.current ?? []),
      })),
      // Apply a schema template to user/group mapping fields
      applyTemplate: assign((context: any, event: any) => {
        const template = event.template as LdapSchemaTemplate;
        if (!template) return {};
        return {
          data: {
            ...context.data,
            userBaseDn: template.userBaseDn ?? context.data.userBaseDn,
            userSubtree: template.userSubtree ?? context.data.userSubtree,
            userObjectClass: template.userObjectClass ?? context.data.userObjectClass,
            userLdapFilter: template.userLdapFilter ?? context.data.userLdapFilter,
            userIdAttribute: template.userIdAttribute ?? context.data.userIdAttribute,
            userRealNameAttribute: template.userRealNameAttribute ?? context.data.userRealNameAttribute,
            userEmailAddressAttribute: template.userEmailAddressAttribute ?? context.data.userEmailAddressAttribute,
            userPasswordAttribute: template.userPasswordAttribute ?? context.data.userPasswordAttribute,
            ldapGroupsAsRoles: template.ldapGroupsAsRoles ?? context.data.ldapGroupsAsRoles,
            groupType: template.groupType ?? context.data.groupType,
            groupBaseDn: template.groupBaseDn ?? context.data.groupBaseDn,
            groupSubtree: template.groupSubtree ?? context.data.groupSubtree,
            groupObjectClass: template.groupObjectClass ?? context.data.groupObjectClass,
            groupIdAttribute: template.groupIdAttribute ?? context.data.groupIdAttribute,
            groupMemberAttribute: template.groupMemberAttribute ?? context.data.groupMemberAttribute,
            groupMemberFormat: template.groupMemberFormat ?? context.data.groupMemberFormat,
            userMemberOfAttribute: template.userMemberOfAttribute ?? context.data.userMemberOfAttribute,
          },
        };
      }),
      // Confirm the re-entered password (edit mode) and immediately move to
      // validation/submission in the same synchronous transition — no
      // setTimeout needed, since XState's send() processes the transition
      // (including entry actions like 'validate') before returning.
      confirmPasswordAndSubmit: assign((context: any, event: any) => ({
        data: { ...context.data, authPassword: event.value as string },
      })),
      // Handle protocol change with automatic port update
      changeProtocol: assign((context: any, event: any) => {
        const protocol = event.value as string;
        let port = context.data.port;
        if (protocol === 'ldaps' && port === 389) {
          port = 636;
        } else if (protocol === 'ldap' && port === 636) {
          port = 389;
        }
        return {
          data: { ...context.data, protocol, port },
          touched: { ...context.touched, protocol: true },
        };
      }),
    },
    services: {
      load: async () => {
        // Build initial form data from server or defaults (shared helper — see serverToFormData)
        return {
          data: serverToFormData(preloadedServer),
          server: preloadedServer ?? null,
          templates: [] as LdapSchemaTemplate[],
        };
      },
      // save service is provided via useForm options
    },
    // Custom event: apply a schema template to user/group mapping fields
    on: {
      APPLY_TEMPLATE: {
        actions: ['applyTemplate', 'validate', 'computePristine'],
      },
      PROTOCOL_CHANGE: {
        actions: ['changeProtocol', 'validate', 'computePristine'],
      },
      // Assign the re-entered password, then go straight to validating/submitting
      // (mirrors the base machine's SUBMIT transition target). Uses the array
      // form for consistency with every other target-bearing custom event in
      // this codebase (see privilegeFormMachine.ts's TYPE_CHANGE,
      // sslFormMachine.ts's SOURCE_CHANGE) - FormMachineConfig['on'] only
      // permits a bare { actions } object when there is no target.
      CONFIRM_PASSWORD_AND_SUBMIT: [
        {
          target: 'validating',
          actions: ['confirmPasswordAndSubmit'],
        },
      ],
    },
  });
}
