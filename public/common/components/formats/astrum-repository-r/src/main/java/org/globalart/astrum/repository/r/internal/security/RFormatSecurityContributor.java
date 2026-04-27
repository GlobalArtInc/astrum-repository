
package org.globalart.astrum.repository.r.internal.security;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.r.RFormat;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
public class RFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public RFormatSecurityContributor(@Qualifier(RFormat.NAME) final Format format) {
    super(format);
  }
}
