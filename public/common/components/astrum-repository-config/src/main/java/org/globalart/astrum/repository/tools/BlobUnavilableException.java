
package org.globalart.astrum.repository.tools;

/**
 * Thrown when Blob InputStream reports no available bytes.
 * 
 * @since 3.3
 */
public class BlobUnavilableException
    extends Exception
{
  public BlobUnavilableException() {
    super("Blob InputStream reports unavailable");
  }
}
