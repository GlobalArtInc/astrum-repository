
package org.globalart.astrum.common.conversion;

import java.util.Optional;

public class BooleanPropertyEditor
    extends PropertyEditorSupport
{
  @Override
  public void setValue(final Object value) {
    this.value = Optional.ofNullable((String) value)
        .map(Boolean::valueOf)
        .orElse(null);
  }
}
