
package org.globalart.astrum.repository.apt;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * @since 3.17
 */
@Component
@Qualifier(AptFormat.NAME)
public class AptFormat
    extends Format
{
  public static final String NAME = "apt";

  public AptFormat() {
    super(NAME);
  }
}
