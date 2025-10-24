
package org.globalart.astrum.common.conversion;

import jakarta.inject.Provider;

public class ProviderPropertyEditor
    extends PropertyEditorSupport
{
  @Override
  public void setValue(final Object value) {
    if (value == null) {
      this.value = null;
    }
    this.value = (Provider) () -> value;
  }
}
