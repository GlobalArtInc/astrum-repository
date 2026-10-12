package org.globalart.astrum.repository.swift.internal.security;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;
import org.globalart.astrum.repository.swift.SwiftFormat;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
public class SwiftFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public SwiftFormatSecurityContributor(@Qualifier(SwiftFormat.NAME) final Format format) {
    super(format);
  }
}
