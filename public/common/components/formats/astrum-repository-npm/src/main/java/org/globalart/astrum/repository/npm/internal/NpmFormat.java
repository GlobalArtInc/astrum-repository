
package org.globalart.astrum.repository.npm.internal;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;


import org.globalart.astrum.repository.Format;

/**
 * npm repository format.
 *
 * @since 3.0
 */
@Component
@Qualifier(NpmFormat.NAME)
public class NpmFormat
    extends Format
{
  public static final String NAME = "npm";

  public NpmFormat() {
    super(NAME);
  }
}
