package org.globalart.astrum.repository.nuget.internal.search;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.utils.PreReleaseEvaluator;
import org.globalart.astrum.repository.nuget.internal.NugetFormat;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetVersion;

import org.springframework.beans.factory.annotation.Qualifier;

@org.springframework.stereotype.Component
@Qualifier(NugetFormat.NAME)
public class NugetPreReleaseEvaluator
    implements PreReleaseEvaluator
{
  @Override
  public boolean isPreRelease(final Component component, final Iterable<Asset> assets) {
    return NugetVersion.parse(component.version()).map(NugetVersion::isPrerelease).orElse(false);
  }
}
