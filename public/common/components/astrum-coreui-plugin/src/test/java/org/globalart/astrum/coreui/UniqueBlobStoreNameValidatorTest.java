
package org.globalart.astrum.coreui;

import org.globalart.astrum.blobstore.api.BlobStoreManager;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * Tests {@link UniqueBlobStoreNameValidator}
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class UniqueBlobStoreNameValidatorTest
{
  @Mock
  private BlobStoreManager blobStoreManager;

  private UniqueBlobStoreNameValidator underTest;

  @Before
  public void setup() {
    underTest = new UniqueBlobStoreNameValidator(blobStoreManager);
  }

  @Test
  public void testValidation() {
    when(blobStoreManager.exists("test")).thenReturn(true);
    when(blobStoreManager.exists("DEFAULT")).thenReturn(false);

    assertFalse(underTest.isValid("test", null));
    assertTrue(underTest.isValid("DEFAULT", null));
  }
}
