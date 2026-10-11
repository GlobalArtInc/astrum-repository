package org.globalart.astrum.security.authz;

import org.apache.shiro.authz.Permission;

/**
 * A principal that narrows what its subject may do, whatever the subject's roles grant.
 *
 * {@link ExceptionCatchingModularRealmAuthorizer} denies every permission that a scoped principal of the subject
 * does not permit.
 */
public interface ScopedPrincipal
{
  boolean permits(Permission permission);
}
