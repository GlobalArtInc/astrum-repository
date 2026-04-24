
package org.globalart.astrum.repository.r;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(RFormat.NAME)
public class RFormat
    extends Format
{
  public static final String NAME = "r";

  public RFormat() {
    super(NAME);
  }
}
