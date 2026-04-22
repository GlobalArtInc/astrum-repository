
package org.globalart.astrum.repository.alpine.internal;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
public class AlpineFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public AlpineFormatSecurityContributor(@Qualifier(AlpineFormat.NAME) final Format format) {
    super(format);
  }
}
