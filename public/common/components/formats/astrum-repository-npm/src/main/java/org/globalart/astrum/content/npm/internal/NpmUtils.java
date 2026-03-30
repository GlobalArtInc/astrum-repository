
package org.globalart.astrum.content.npm.internal;

import java.io.ByteArrayInputStream;
import java.io.InputStream;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.npm.internal.NpmJsonUtils;

import static com.google.common.collect.Maps.newHashMap;

/**
 * Shared code for npm.
 *
 * @since 3.next
 */
public final class NpmUtils
{
  private NpmUtils() {
    // nop
  }

  /**
   * Returns a new {@link InputStream} that returns an error object. Mostly useful for NPM Responses that have already
   * been written with a successful status (like a 200) but just before streaming out content found an issue preventing
   * the intended content to be streamed out.
   *
   * @return InputStream
   */
  public static InputStream errorInputStream(final String message) {
    NestedAttributesMap errorObject = new NestedAttributesMap("error", newHashMap());
    errorObject.set("success", false);
    errorObject.set("error", "Failed to stream response due to: " + message);
    return new ByteArrayInputStream(NpmJsonUtils.bytes(errorObject));
  }
}
