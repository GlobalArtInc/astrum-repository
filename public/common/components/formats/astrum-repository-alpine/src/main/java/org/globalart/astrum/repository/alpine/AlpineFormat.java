
package org.globalart.astrum.repository.alpine;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(AlpineFormat.NAME)
public class AlpineFormat
    extends Format
{
  public static final String NAME = "alpine";

  public AlpineFormat() {
    super(NAME);
  }
}
