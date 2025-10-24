
package org.globalart.astrum.common.conversion;

import java.net.URI;
import java.util.Optional;

public class URIPropertyEditor
    extends PropertyEditorSupport
{
  @Override
  public void setValue(final Object value) {
    this.value = Optional.ofNullable((String) value).map(URI::create).orElse(null);
  }
}
