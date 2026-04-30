
package org.globalart.astrum.repository.cocoapods.internal;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
public class CocoapodsFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public CocoapodsFormatSecurityContributor(@Qualifier(CocoapodsFormat.NAME) final Format format) {
    super(format);
  }
}
