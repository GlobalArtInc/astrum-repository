package org.globalart.astrum.repository.rubygems.internal;

import java.util.Set;

import org.globalart.astrum.repository.rest.api.AssetXODescriptor;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;
import org.globalart.astrum.repository.rubygems.internal.gem.GemSpec;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(RubygemsFormat.NAME)
public class RubygemsAssetXODescriptor
    implements AssetXODescriptor
{
  private static final Set<String> ATTRIBUTE_KEYS = Set.of(GemSpec.NAME, GemSpec.VERSION, GemSpec.PLATFORM,
      GemSpec.SUMMARY, GemSpec.DESCRIPTION, GemSpec.AUTHORS, GemSpec.LICENSES, GemSpec.HOMEPAGE);

  @Override
  public Set<String> listExposedAttributeKeys() {
    return ATTRIBUTE_KEYS;
  }
}
