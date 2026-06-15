
package org.globalart.astrum.coreui;

import java.util.List;

/**
 * Proprietary Repositories Settings exchange object.
 *
 * @since 3.30
 */
public class ProprietaryRepositoriesXO
{
  private List<String> enabledRepositories;

  public List<String> getEnabledRepositories() {
    return enabledRepositories;
  }

  public void setEnabledRepositories(List<String> enabledRepositories) {
    this.enabledRepositories = enabledRepositories;
  }

  @Override
  public String toString() {
    return "ProprietaryRepositoriesXO{" +
        "enabledRepositories=" + enabledRepositories +
        '}';
  }
}
