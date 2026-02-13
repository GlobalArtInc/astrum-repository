
package org.globalart.astrum.repository.content.store.internal.migration;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Tests for {@link AssetBlobRefMigrationTaskDescriptor}
 */
public class AssetBlobRefMigrationTaskDescriptorTest
{
  @Test
  void testTaskIsRecoverable() {
    AssetBlobRefMigrationTaskDescriptor descriptor = new AssetBlobRefMigrationTaskDescriptor();
    assertTrue(descriptor.isRecoverable(), "Migration task should be recoverable to handle interruptions");
  }
}
