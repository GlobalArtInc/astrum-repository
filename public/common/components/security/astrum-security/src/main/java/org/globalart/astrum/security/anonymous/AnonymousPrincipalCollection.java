
package org.globalart.astrum.security.anonymous;

import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.SimplePrincipalCollection;
import org.apache.shiro.subject.Subject;

// NOTE: This apparently needs to be non-internal for class-loading accessibility

/**
 * Anonymous user {@link PrincipalCollection}.
 *
 * Used to help identify if a {@link Subject} is anonymous.
 *
 * @since 3.0
 */
public class AnonymousPrincipalCollection
    extends SimplePrincipalCollection
{
  public AnonymousPrincipalCollection(final Object principal, final String realmName) {
    super(principal, realmName);
  }
}
