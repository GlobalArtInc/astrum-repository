
package org.globalart.astrum.security.authz;

/**
 * Resource info collects the description of HOW and WHAT has been accessed.
 */
public class ResourceInfo
{
  private final String accessProtocol;

  private final String accessMethod;

  private final String action;

  private final String accessedUri;

  public ResourceInfo(
      final String accessProtocol,
      final String accessMethod,
      final String action,
      final String accessedUri)
  {
    this.accessProtocol = accessProtocol;
    this.accessMethod = accessMethod;
    this.action = action;
    this.accessedUri = accessedUri;
  }

  public String getAccessProtocol() {
    return accessProtocol;
  }

  public String getAccessMethod() {
    return accessMethod;
  }

  public String getAction() {
    return action;
  }

  public String getAccessedUri() {
    return accessedUri;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }

    ResourceInfo that = (ResourceInfo) o;

    if (accessMethod != null ? !accessMethod.equals(that.accessMethod) : that.accessMethod != null) {
      return false;
    }
    if (accessProtocol != null ? !accessProtocol.equals(that.accessProtocol) : that.accessProtocol != null) {
      return false;
    }
    if (accessedUri != null ? !accessedUri.equals(that.accessedUri) : that.accessedUri != null) {
      return false;
    }
    if (action != null ? !action.equals(that.action) : that.action != null) {
      return false;
    }

    return true;
  }

  @Override
  public int hashCode() {
    int result = accessProtocol != null ? accessProtocol.hashCode() : 0;
    result = 31 * result + (accessMethod != null ? accessMethod.hashCode() : 0);
    result = 31 * result + (action != null ? action.hashCode() : 0);
    result = 31 * result + (accessedUri != null ? accessedUri.hashCode() : 0);
    return result;
  }
}
