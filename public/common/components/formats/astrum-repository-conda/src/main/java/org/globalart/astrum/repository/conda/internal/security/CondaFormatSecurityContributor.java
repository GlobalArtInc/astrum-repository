
package org.globalart.astrum.repository.conda.internal.security;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import org.globalart.astrum.repository.conda.CondaFormat;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

@Component
public class CondaFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public CondaFormatSecurityContributor(@Qualifier(CondaFormat.NAME) final Format format) {
    super(format);
  }
}
