
package org.globalart.astrum.repository.golang.internal.security;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.golang.GolangFormat;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

@Component
public class GolangFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public GolangFormatSecurityContributor(@Qualifier(GolangFormat.NAME) final Format format) {
    super(format);
  }
}
