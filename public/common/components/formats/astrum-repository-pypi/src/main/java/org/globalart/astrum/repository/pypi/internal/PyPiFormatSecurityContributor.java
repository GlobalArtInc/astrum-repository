
package org.globalart.astrum.repository.pypi.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;


import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

/**
 * PyPI format security contributor.
 *
 * @since 3.1
 */
@Component
public class PyPiFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public PyPiFormatSecurityContributor(@Qualifier(PyPiFormat.NAME) final Format format) {
    super(format);
  }
}
