
package org.globalart.astrum.blobstore.api;

import java.time.OffsetDateTime;

/**
 * Metadata associated with a remotely stored blob
 */
public record ExternalMetadata(String etag, OffsetDateTime lastModified)
{
}
