
package org.globalart.astrum.repository.golang;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import org.globalart.astrum.repository.Format;

@Component
@Qualifier(GolangFormat.NAME)
public class GolangFormat
    extends Format
{
  public static final String NAME = "go";

  public GolangFormat() {
    super(NAME);
  }
}
