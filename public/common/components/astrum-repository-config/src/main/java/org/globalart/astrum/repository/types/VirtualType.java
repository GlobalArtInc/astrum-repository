
package org.globalart.astrum.repository.types;

import org.globalart.astrum.repository.Type;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Virtual repository type.
 *
 * @since 3.0
 */
@Component
@Qualifier(VirtualType.NAME)
public class VirtualType
    extends Type
{
  public static final String NAME = "virtual";

  @VisibleForTesting
  public VirtualType() {
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
