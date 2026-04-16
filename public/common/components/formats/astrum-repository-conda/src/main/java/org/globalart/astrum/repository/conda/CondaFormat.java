
package org.globalart.astrum.repository.conda;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import org.globalart.astrum.repository.Format;

@Component
@Qualifier(CondaFormat.NAME)
public class CondaFormat
    extends Format
{
  public static final String NAME = "conda";

  public CondaFormat() {
    super(NAME);
  }
}
