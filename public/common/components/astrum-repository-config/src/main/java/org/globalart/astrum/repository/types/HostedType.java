
package org.globalart.astrum.repository.types;

import org.globalart.astrum.repository.Type;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Hosted repository type.
 *
 * @since 3.0
 */
@Component
@Qualifier(HostedType.NAME)
public class HostedType
    extends Type
{
  public static final String NAME = "hosted";

  @VisibleForTesting
  public HostedType() {
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
