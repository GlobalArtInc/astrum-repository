
package org.globalart.astrum.repository.firewall.event;

import java.util.List;
import java.util.concurrent.CompletableFuture;

import org.globalart.astrum.repository.Repository;

/**
 * @since 3.35
 */
public class QuarantineComponentsRequestEvent
{
  private final CompletableFuture<List<String>> result = new CompletableFuture<>();

  private final Repository repository;

  private final List<FirewallComponent> components;

  public QuarantineComponentsRequestEvent(
      final Repository repository,
      final List<FirewallComponent> components)
  {
    this.repository = repository;
    this.components = components;
  }

  public CompletableFuture<List<String>> getResult() {
    return result;
  }

  public Repository getRepository() {
    return repository;
  }

  public List<FirewallComponent> getComponents() {
    return components;
  }

  public static class FirewallComponent
  {
    private final String hash;

    private final String path;

    private final String version;

    public FirewallComponent(final String hash, final String path, final String version) {
      this.hash = hash;
      this.path = path;
      this.version = version;
    }

    public String getHash() {
      return hash;
    }

    public String getPath() {
      return path;
    }

    public String getVersion() {
      return version;
    }
  }
}
