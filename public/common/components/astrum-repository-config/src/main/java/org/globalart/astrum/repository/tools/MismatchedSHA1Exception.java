
package org.globalart.astrum.repository.tools;

/**
 * Exception thrown when Asset record metadata has a different SHA1 than the blob suggests.
 * 
 * @since 3.3
 */
public class MismatchedSHA1Exception
    extends Exception
{
  public MismatchedSHA1Exception() {
    super("SHA1 values disagree between the DB and the BlobStore");
  }
}
