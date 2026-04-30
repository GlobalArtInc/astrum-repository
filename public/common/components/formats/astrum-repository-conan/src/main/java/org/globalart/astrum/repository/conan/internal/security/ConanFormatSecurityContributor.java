
package org.globalart.astrum.repository.conan.internal.security;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.conan.ConanFormat;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
public class ConanFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public ConanFormatSecurityContributor(@Qualifier(ConanFormat.NAME) final Format format) {
    super(format);
  }
}
