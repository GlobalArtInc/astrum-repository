package org.globalart.astrum.repository.yum.internal.search;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.utils.PreReleaseEvaluator;
import org.globalart.astrum.repository.yum.YumFormat;

import org.springframework.beans.factory.annotation.Qualifier;

@org.springframework.stereotype.Component
@Qualifier(YumFormat.NAME)
public class YumPreReleaseEvaluator
    implements PreReleaseEvaluator
{
  @Override
  public boolean isPreRelease(final Component component, final Iterable<Asset> assets) {
    return isPreRelease(component.version());
  }

  public static boolean isPreRelease(final String componentVersion) {
    if (componentVersion == null || componentVersion.isEmpty()) {
      return false;
    }
    String versionRelease = componentVersion.substring(componentVersion.indexOf(':') + 1);
    int dash = versionRelease.lastIndexOf('-');
    String version = dash < 0 ? versionRelease : versionRelease.substring(0, dash);
    String release = dash < 0 ? "" : versionRelease.substring(dash + 1);
    return version.contains("~") || release.startsWith("0.");
  }
}
