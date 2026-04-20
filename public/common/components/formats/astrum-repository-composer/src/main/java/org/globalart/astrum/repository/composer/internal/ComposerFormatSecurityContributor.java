
package org.globalart.astrum.repository.composer.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;


import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.composer.ComposerFormat;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

/**
 * Boilerplate security contributor for the Composer format.
 */
@Component
public class ComposerFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public ComposerFormatSecurityContributor(@Qualifier(ComposerFormat.NAME) final Format format) {
    super(format);
  }
}
