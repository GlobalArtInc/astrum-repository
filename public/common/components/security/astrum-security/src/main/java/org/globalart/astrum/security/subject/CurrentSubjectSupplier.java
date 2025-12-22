
package org.globalart.astrum.security.subject;

import java.util.function.Supplier;
import org.apache.shiro.SecurityUtils;
import org.apache.shiro.subject.Subject;

/**
 * Current subject supplier.
 *
 * @since 2.6
 */
public class CurrentSubjectSupplier
    implements Supplier<Subject>
{
  @Override
  public Subject get() {
    return SecurityUtils.getSubject();
  }
}
