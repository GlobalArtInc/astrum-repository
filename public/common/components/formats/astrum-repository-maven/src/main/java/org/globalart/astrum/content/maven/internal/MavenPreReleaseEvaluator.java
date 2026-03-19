
package org.globalart.astrum.content.maven.internal;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.utils.PreReleaseEvaluator;
import org.globalart.astrum.repository.maven.internal.Maven2Format;

import static org.globalart.astrum.repository.maven.internal.Attributes.P_BASE_VERSION;
import static org.globalart.astrum.repository.maven.internal.Constants.SNAPSHOT_VERSION_SUFFIX;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * @since 3.38
 */
@org.springframework.stereotype.Component
@Qualifier(Maven2Format.NAME)
public class MavenPreReleaseEvaluator
    implements PreReleaseEvaluator
{

  @Override
  public boolean isPreRelease(final FluentComponent component) {
    return isPreRelease((Component) component);
  }

  @Override
  public boolean isPreRelease(final Component component, final Iterable<Asset> assets) {
    return isPreRelease(component);
  }

  private static boolean isPreRelease(final Component component) {
    String baseVersion = component.attributes().child(Maven2Format.NAME).get(P_BASE_VERSION, String.class);
    if (baseVersion == null) {
      return false;
    }
    return baseVersion.endsWith(SNAPSHOT_VERSION_SUFFIX);
  }
}
