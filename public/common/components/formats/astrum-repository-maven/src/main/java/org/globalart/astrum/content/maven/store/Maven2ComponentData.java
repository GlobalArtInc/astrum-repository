

package org.globalart.astrum.content.maven.store;

import org.globalart.astrum.repository.content.store.ComponentData;

/**
 * @since 3.29
 */
public class Maven2ComponentData
    extends ComponentData
{
  private String baseVersion;

  public String getBaseVersion() {
    return baseVersion;
  }

  public void setBaseVersion(final String baseVersion) {
    this.baseVersion = baseVersion;
  }
}
