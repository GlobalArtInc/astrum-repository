
package org.globalart.astrum.repository.browse.node;

import java.util.Objects;

import static java.lang.String.format;

/**
 * Used to denote a node's request path (used for security checks) and a potentially different display name.
 * Suppose when browsing format 'dash' we change all folders named 'dash' to '-' for display purposes, i.e.
 *
 * <pre>
 * requestPath         | displayName
 * --------------------+----------------------
 * /org/                | org
 * /org/foo/            | foo
 * /org/foo/-/          | dash
 * /org/foo/-/some.file | some.file
 * </pre>
 *
 * @since 3.18
 */
public class BrowsePath
{
  public static final char SLASH_CHAR = '/';

  public static final String SLASH = "/";

  private String displayName;

  private String requestPath;

  public BrowsePath(String displayName, String requestPath) {
    this.displayName = displayName;
    this.requestPath = requestPath;
  }

  public String getDisplayName() {
    return displayName;
  }

  public String getRequestPath() {
    return requestPath;
  }

  public void setDisplayName(final String displayName) {
    this.displayName = displayName;
  }

  public void setRequestPath(final String requestPath) {
    this.requestPath = requestPath;
  }

  @Override
  public boolean equals(final Object obj) {
    if (!(obj instanceof BrowsePath)) {
      return false;
    }

    return Objects.equals(displayName, ((BrowsePath) obj).getDisplayName())
        && Objects.equals(requestPath, ((BrowsePath) obj).getRequestPath());
  }

  @Override
  public int hashCode() {
    return Objects.hash(displayName, requestPath);
  }

  @Override
  public String toString() {
    return format("Display Name:%s, Request Path:%s", displayName, requestPath);
  }
}
