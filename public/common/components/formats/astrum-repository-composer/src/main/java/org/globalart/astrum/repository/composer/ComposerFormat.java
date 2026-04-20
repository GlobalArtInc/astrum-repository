
package org.globalart.astrum.repository.composer;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;


import org.globalart.astrum.repository.Format;

/**
 * Class representing the Composer format.
 */
@Component
@Qualifier(ComposerFormat.NAME)
public class ComposerFormat
    extends Format
{
  public static final String NAME = "composer";

  public ComposerFormat() {
    super(NAME);
  }
}
