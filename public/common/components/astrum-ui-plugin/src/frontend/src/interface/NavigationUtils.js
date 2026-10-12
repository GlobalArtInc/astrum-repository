

import ExtJS from "./ExtJS";

/**
 * This is used to check the visibility of route defined with UIRouter. It expects a visibilityRequirements block
 *
 * visibilityRequirements may have any of the following:
 *
 *   bundleActive: string
 *   licenseValid: { key: string, defaultValue: boolean } []
 *   statesEnabled: { key: string, defaultValue: boolean } []
 *   permissions : string []
 *   requiresPermission: string
 *   requiresAnyPermission: string []
 *   permissionPrefix: string
 *   permissionPrefixes: string []
 *   editions: string []
 *   requiresUser: boolean
 *
 * @param visibilityRequirements
 * @returns {boolean}
 */
export function isVisible(visibilityRequirements) {
  if (!visibilityRequirements) {
    return true;
  }

  const {
    bundle,
    licenseValid,
    statesEnabled,
    permissions,
    requiresPermission,
    requiresAnyPermission,
    permissionPrefix,
    permissionPrefixes,
    capability,
    editions,
    requiresUser,
    notClustered,
    anonymousAccessOrHasUser
  } = visibilityRequirements;

  // check that all our expected global dependencies are in place
  // If ExtJS isn't ready yet, check for session cookie to determine visibility
  const depsValid = hasValidDependencies();
  if (!depsValid) {
    // If no visibility requirements exist, allow the route (for login, etc.)
    const hasAnyRequirements = bundle || licenseValid || statesEnabled || permissions ||
                                requiresPermission || requiresAnyPermission || permissionPrefix ||
                                permissionPrefixes || capability || editions || requiresUser ||
                                notClustered || anonymousAccessOrHasUser;
    if (!hasAnyRequirements) {
      return true;
    }
    // CRITICAL FIX for debug mode refresh: If user has session cookie, assume they're authorized
    // until ExtJS loads and can properly check permissions. This prevents logout on refresh.
    if (document.cookie.includes('ASTRUMSESSIONID')) {
      return true;
    }
    return false;
  }

  const Application = AS?.app?.Application;
  const Security = AS?.Security;

  // hide this route on HA
  if (notClustered && isClustered()) {
    return false;
  }

  if (bundle && !Application.bundleActive(bundle)) {
    // check that the bundles required by the route are enabled
    return false;
  }

  // check that all licenses required by this are present
  if (licenseValid && !areAllRequiredLicensesPresent(licenseValid)) {
    return false;
  }

  // check that all required statesEnabled for this route are present
  if (statesEnabled && !areAllRequiredStatesEnabled(statesEnabled)) {
    return false;
  }

  // check that all required permissions are present
  if (permissions && !areAllRequiredPermissionsPresent(permissions)) {
    return false
  }

  // check a single required permission (convenience shorthand for a single permission string)
  if (requiresPermission && !AS.Permissions.check(requiresPermission)) {
    return false;
  }

  // check that at least one of the listed permissions is present
  if (requiresAnyPermission && !hasAnyOfTheRequiredPermissions(requiresAnyPermission)) {
    return false;
  }

  if (permissionPrefix && !hasAnyPermissionWithPrefix(permissionPrefix)) {
    return false;
  }

  // check that user has ANY permission matching at least one of the prefixes
  if (permissionPrefixes && !hasAnyPermissionWithAnyPrefix(permissionPrefixes)) {
    return false;
  }

  // check that edition requirements are met, i.e. must be PRO or COMMUNITY
  if (editions && !meetsEditionRequirement(editions)) {
    return false;
  }

  // check if the required capability is enabled and active
  if (capability && !isTheRequiredCapabilityPresentAndActive(capability)) {
    return false;
  }

  if (requiresUser && !Security.hasUser() && !hasAnonymousPermissions()) {
    return false
  }

  if (anonymousAccessOrHasUser &&
      !(!!AS.State.getValue('anonymousUsername') || Security.hasUser())) {
    return false;
  }

  return true;
}

function areAllRequiredStatesEnabled(statesEnabled) {
  return statesEnabled.every(state => {
    const stateValue = AS.State.getValue(state.key, state.defaultValue);

    if (typeof stateValue === "boolean") {
      return stateValue;
    }
    else if (Array.isArray(stateValue)) {
      return stateValue.length > 0;
    }
    else {
      return stateValue?.enabled ?? false;
    }
  });
}

function areAllRequiredPermissionsPresent(permissions) {
  return permissions.every((permission) => {
    const hasPermission = AS.Permissions.check(permission);
    return hasPermission;
  });
}

/**
 * Check if user has ANY of the listed permissions.
 * Returns true if at least one permission is present.
 *
 * @param {string[]} permissions - Array of permissions to check
 * @returns {boolean} True if user has any of the permissions
 */
function hasAnyOfTheRequiredPermissions(permissions) {
  if (!AS.Permissions) {
    return false;
  }

  for (const permission of permissions) {
    if (AS.Permissions.check(permission)) {
      return true;
    }
  }

  return false;
}

/**
 * Check if user has ANY permission starting with the given prefix.
 * This replicates the behavior of AS.Permissions.checkExistsWithPrefix from the older version.
 *
 * @param {string} prefix - The permission prefix to check
 * @returns {boolean} True if user has any permission starting with the prefix
 */
function hasAnyPermissionWithPrefix(prefix) {
  if (!AS.Permissions || !AS.Permissions.permissions) {
    return false;
  }

  const permissions = AS.Permissions.permissions;

  // Check if any permission starts with the prefix and is permitted
  for (const permission in permissions) {
    if (permission.startsWith(prefix) && permissions[permission] === true) {
      return true;
    }
  }

  return false;
}

/**
 * Check if user has ANY permission starting with ANY of the given prefixes.
 * This is useful for routes that should be visible with multiple types of permissions,
 * such as Browse which should be visible for both repository-view and repository-content-selector permissions.
 *
 * @param {string[]} prefixes - Array of permission prefixes to check
 * @returns {boolean} True if user has any permission starting with any of the prefixes
 */
function hasAnyPermissionWithAnyPrefix(prefixes) {
  if (!prefixes || prefixes.length === 0) {
    return false;
  }

  if (!AS.Permissions || !AS.Permissions.permissions) {
    return false;
  }

  // Check if any permission matches any of the prefixes
  for (const prefix of prefixes) {
    if (hasAnyPermissionWithPrefix(prefix)) {
      return true;
    }
  }

  return false;
}

function areAllRequiredLicensesPresent(licenseValid) {
  return licenseValid.every(
      licenseValid => {
        const value = AS.State.getValue(licenseValid.key, licenseValid.defaultValue)
        if (typeof value !== 'object') {
          return false;
        }

        return value.licenseValid;
      });
}

function isTheRequiredCapabilityPresentAndActive(capability) {
  const activeTypes = AS.State.getValue('capabilityActiveTypes') || [];
  const createdTypes = AS.State.getValue('capabilityCreatedTypes') || [];

  return activeTypes.includes(capability) && createdTypes.includes(capability);
}

function meetsEditionRequirement(editions) {
  return editions.some((edition) => {
    return AS.State.getEdition() === edition;
  });
}

function isClustered() {
  return !!AS.State.getValue('astrum.datastore.clustered.enabled');
}

function hasValidDependencies() {
  const Application = AS.app?.Application;
  const State = AS.State;
  const Permissions = AS.Permissions;
  const Security = AS.Security;

  // AS.getApplication is attached by ExtJS only after Ext.app.Application
  // is instantiated (initNamespace). Without it, AS.State.getValue() will
  // throw "AS.getApplication is not a function" because internally it calls
  // AS.getApplication().getStateController(). Guard here so React routes that
  // run before ExtJS finishes booting fall through to the cookie/no-deps path.
  const appReady =
      typeof window.AS?.getApplication === 'function' &&
      !!window.AS.getApplication();

  return !!(Application && State && Permissions && Security && appReady);
}

// A `requiresUser` route should also be visible to an anonymous subject that has
// been granted permissions (e.g. anonymous assigned as-admin — ASTRUM-47114).
// Per-route permission checks above already filter routes the anonymous user
// cannot access, so this only re-opens routes the anonymous user genuinely has
// rights to.
//
// Permissions live at AS.Permissions.permissions (a flat map of id → permitted),
// populated by AS.controller.Permissions from rapture_Security.getPermissions.
// They are NOT contributed to AS.State — SecurityComponent.getState() only
// contributes `user` and `anonymousUsername`.
function hasAnonymousPermissions() {
  const Security = AS?.Security;
  if (!Security || Security.hasUser()) {
    return false;
  }
  const perms = AS?.Permissions?.permissions;
  return !!perms && Object.keys(perms).length > 0;
}

export function useIsVisible(visibilityRequirements) {
  return ExtJS.useVisiblityWithChanges(() => isVisible(visibilityRequirements));
}
