
package org.globalart.astrum.repository.apt.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

import org.springframework.beans.factory.annotation.Qualifier;

/**
 * @since 3.17
 */
@Component
public class AptFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public AptFormatSecurityContributor(@Qualifier(AptFormat.NAME) final Format format) {
    super(format);
  }
}
