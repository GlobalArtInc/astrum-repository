
package org.globalart.astrum.repository.maven.internal;

import org.globalart.astrum.repository.Format;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Maven 2 format.
 *
 * @since 3.0
 */
@Component
@Qualifier(Maven2Format.NAME)
public class Maven2Format
    extends Format
{
  public static final String NAME = "maven2";

  public Maven2Format() {
    super(NAME);
  }
}
