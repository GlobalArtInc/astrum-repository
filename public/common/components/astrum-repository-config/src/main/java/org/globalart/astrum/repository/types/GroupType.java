
package org.globalart.astrum.repository.types;

import org.globalart.astrum.repository.Type;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Group repository type.
 *
 * @since 3.0
 */
@Component
@Qualifier(GroupType.NAME)
public class GroupType
    extends Type
{
  public static final String NAME = "group";

  @VisibleForTesting
  public GroupType() {
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
