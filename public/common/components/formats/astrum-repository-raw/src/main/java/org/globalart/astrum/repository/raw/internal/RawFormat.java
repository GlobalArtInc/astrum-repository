
package org.globalart.astrum.repository.raw.internal;

import org.globalart.astrum.repository.Format;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * RAW repository format.
 *
 * @since 3.0
 */
@Component
@Qualifier(RawFormat.NAME)
public class RawFormat
    extends Format
{
  public static final String NAME = "raw";

  public RawFormat() {
    super(NAME);
  }
}
