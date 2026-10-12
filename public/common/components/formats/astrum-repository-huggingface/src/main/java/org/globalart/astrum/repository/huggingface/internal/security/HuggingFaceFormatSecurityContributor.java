package org.globalart.astrum.repository.huggingface.internal.security;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.huggingface.HuggingFaceFormat;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
public class HuggingFaceFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public HuggingFaceFormatSecurityContributor(@Qualifier(HuggingFaceFormat.NAME) final Format format) {
    super(format);
  }
}
