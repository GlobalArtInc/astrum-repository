package org.globalart.astrum.repository.rubygems.internal.security;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
public class RubygemsFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public RubygemsFormatSecurityContributor(@Qualifier(RubygemsFormat.NAME) final Format format) {
    super(format);
  }
}
