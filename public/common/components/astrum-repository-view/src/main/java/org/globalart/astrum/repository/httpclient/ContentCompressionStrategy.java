
package org.globalart.astrum.repository.httpclient;

/**
 * Strategy to disable in the HttpClient automatic content compression/decompression
 *
 * @since 3.27
 */
public interface ContentCompressionStrategy
{
  boolean shouldDisableContentCompression(String repositoryName);
}
