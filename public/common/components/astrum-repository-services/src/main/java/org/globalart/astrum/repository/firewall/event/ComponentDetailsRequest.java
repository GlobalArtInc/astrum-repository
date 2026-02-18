
package org.globalart.astrum.repository.firewall.event;

import java.util.Date;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;

import org.globalart.goodies.packageurl.PackageUrl;

/**
 * @since 3.29
 */
public class ComponentDetailsRequest
{
  private final CompletableFuture<Map<String, Date>> result = new CompletableFuture<>();

  private final String name;

  private final List<PackageUrl> packageUrls;

  public ComponentDetailsRequest(final String name, final List<PackageUrl> packageUrls) {
    this.name = name;
    this.packageUrls = packageUrls;
  }

  public CompletableFuture<Map<String, Date>> getResult() {
    return result;
  }

  public List<PackageUrl> getPackageUrls() {
    return packageUrls;
  }

  public String getName() {
    return name;
  }
}
