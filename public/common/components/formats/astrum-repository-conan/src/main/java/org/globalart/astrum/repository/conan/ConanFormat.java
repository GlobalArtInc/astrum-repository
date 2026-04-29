
package org.globalart.astrum.repository.conan;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(ConanFormat.NAME)
public class ConanFormat
    extends Format
{
  public static final String NAME = "conan";

  public ConanFormat() {
    super(NAME);
  }
}
