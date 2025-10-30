
package org.globalart.astrum.common.time;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;

/**
 * UTC related helper methods.
 * 
 * @since 3.24
 */
public class UTC
{
  private UTC() {
    // empty
  }

  public static OffsetDateTime now() {
    return OffsetDateTime.now(ZoneOffset.UTC);
  }
}
