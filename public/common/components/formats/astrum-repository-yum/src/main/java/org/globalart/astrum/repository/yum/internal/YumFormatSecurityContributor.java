package org.globalart.astrum.repository.yum.internal;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;
import org.globalart.astrum.repository.yum.YumFormat;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
public class YumFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public YumFormatSecurityContributor(@Qualifier(YumFormat.NAME) final Format format) {
    super(format);
  }
}
