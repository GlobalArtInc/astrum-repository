
package org.globalart.astrum.repository.npm.internal;

/**
 * Internal exception thrown when resolving of tarball name to package version using package metadata fails.
 *
 * @since 3.0
 *
 */
public class NonResolvableTarballNameException
    extends RuntimeException {
  public NonResolvableTarballNameException(final String message) {
    super(message);
  }
}
