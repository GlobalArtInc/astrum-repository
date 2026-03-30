
package org.globalart.astrum.repository.npm.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;


import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

/**
 * NPM format security contributor.
 *
 * @since 3.0
 */
@Component
public class NpmFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public NpmFormatSecurityContributor(@Qualifier(NpmFormat.NAME) final Format format) {
    super(format);
  }
}
