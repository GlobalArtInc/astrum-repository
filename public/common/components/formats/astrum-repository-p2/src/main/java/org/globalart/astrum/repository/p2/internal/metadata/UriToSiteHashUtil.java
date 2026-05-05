
package org.globalart.astrum.repository.p2.internal.metadata;

import java.net.URI;
import java.nio.charset.StandardCharsets;

import com.google.common.hash.Hashing;

public class UriToSiteHashUtil
{
  private UriToSiteHashUtil() {
  }

  public static String map(final String url) {
    return Hashing.sha256().hashString(url.endsWith("/") ? url : url + '/', StandardCharsets.UTF_8).toString();
  }

  public static String map(final URI url) {
    return map(url.toString());
  }
}
