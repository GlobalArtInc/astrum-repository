
package org.globalart.astrum.repository.pypi.internal;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;


import org.globalart.astrum.repository.Format;

/**
 * PyPI repository format.
 *
 * @since 3.1
 */
@Component
@Qualifier(PyPiFormat.NAME)
public class PyPiFormat
    extends Format
{
  public static final String NAME = "pypi";

  public PyPiFormat() {
    super(NAME);
  }
}
