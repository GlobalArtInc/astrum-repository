
package org.globalart.astrum.common.conversion;

import java.util.Optional;

import org.globalart.astrum.common.io.ByteSize;

public class ByteSizePropertyEditor
    extends PropertyEditorSupport
{
  @Override
  public void setValue(final Object value) {
    this.value = Optional.ofNullable(value)
        .map(String.class::cast)
        .map(ByteSize::parse)
        .orElse(null);
  }
}
