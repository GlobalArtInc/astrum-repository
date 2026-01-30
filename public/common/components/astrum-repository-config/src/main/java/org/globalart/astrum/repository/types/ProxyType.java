
package org.globalart.astrum.repository.types;

import org.globalart.astrum.repository.Type;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Proxy repository type.
 *
 * @since 3.0
 */
@Component
@Qualifier(ProxyType.NAME)
public class ProxyType
    extends Type
{
  public static final String NAME = "proxy";

  @VisibleForTesting
  public ProxyType() {
    super(NAME);
  }

  @Override
  public Class<?> getValidationGroup() {
    return ValidationGroup.class;
  }

  public interface ValidationGroup
  {
    // empty
  }
}
