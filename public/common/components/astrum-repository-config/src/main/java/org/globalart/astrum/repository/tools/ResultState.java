
package org.globalart.astrum.repository.tools;

/**
 * Possible error states that can be detected by {@link DeadBlobFinder}.
 * 
 * @since 3.3
 */
public enum ResultState
{
  ASSET_DELETED, // DB record was deleted during inspection
  DELETED, // DB record references blobRef which has since been deleted
  MISSING_BLOB_REF, // DB record has no blobRef
  SHA1_DISAGREEMENT, // DB record and blob have different SHA1
  UNAVAILABLE_BLOB, // blob has an inputstream that reports 0 when calling isAvailable()
  UNKNOWN,
  UNREADABLE_BLOB, // Unable to read blob from disk
}
