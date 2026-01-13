
package org.globalart.astrum.audit;

import javax.annotation.Nonnull;

/**
 * Provides the current context {@link AuditData#initiator} value.
 *
 * @since 3.1
 */
public interface InitiatorProvider
{
  @Nonnull
  String get();
}
