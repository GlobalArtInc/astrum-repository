
package org.globalart.astrum.common.conversion;

import java.util.Optional;

import org.globalart.astrum.common.time.Time;

public class TimePropertyEditor
    extends PropertyEditorSupport
{
  @Override
  public void setValue(final Object value) {
    this.value = Optional.ofNullable(value)
        .map(String.class::cast)
        .map(Time::parse)
        .orElse(null);
  }
}
