
package org.globalart.astrum.common.conversion;

import java.io.File;
import java.util.Optional;

public class FilePropertyEditor
    extends PropertyEditorSupport
{
  @Override
  public void setValue(final Object value) {
    this.value = Optional.ofNullable((String) value)
        .map(File::new)
        .orElse(null);
  }
}
