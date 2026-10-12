package org.globalart.astrum.repository.rubygems.internal.cleanup;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.utils.PreReleaseEvaluator;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;
import org.globalart.astrum.repository.rubygems.internal.gem.GemVersion;

import org.springframework.beans.factory.annotation.Qualifier;

@org.springframework.stereotype.Component
@Qualifier(RubygemsFormat.NAME)
public class RubygemsPreReleaseEvaluator
    implements PreReleaseEvaluator
{
  @Override
  public boolean isPreRelease(final Component component, final Iterable<Asset> assets) {
    return GemVersion.isPrerelease(component.version());
  }
}
