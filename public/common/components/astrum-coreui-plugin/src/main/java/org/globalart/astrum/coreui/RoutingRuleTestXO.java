
package org.globalart.astrum.coreui;

import java.util.List;
import jakarta.validation.constraints.NotBlank;

import org.globalart.astrum.repository.routing.RoutingMode;

/**
 * Routing Rule Test transfer object for internal REST API.
 *
 * @since 3.16
 */
public class RoutingRuleTestXO
{
  @NotBlank
  private RoutingMode mode;

  @NotBlank
  private List<String> matchers;

  @NotBlank
  private String path;

  public RoutingMode getMode() {
    return mode;
  }

  public void setMode(RoutingMode mode) {
    this.mode = mode;
  }

  public List<String> getMatchers() {
    return matchers;
  }

  public void setMatchers(List<String> matchers) {
    this.matchers = matchers;
  }

  public String getPath() {
    return path;
  }

  public void setPath(String path) {
    this.path = path;
  }

}
