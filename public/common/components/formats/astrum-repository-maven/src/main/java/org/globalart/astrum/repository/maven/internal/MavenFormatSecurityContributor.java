
package org.globalart.astrum.repository.maven.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Maven 2 format security contributor.
 *
 * @since 3.0
 */
@Component
public class MavenFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public MavenFormatSecurityContributor(@Qualifier(Maven2Format.NAME) final Format format) {
    super(format);
  }
}
