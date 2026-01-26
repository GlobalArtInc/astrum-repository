
package org.globalart.astrum.blobstore;

import org.globalart.astrum.blobstore.api.BlobId;

import org.junit.Before;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.globalart.astrum.blobstore.DirectPathLocationStrategy.DIRECT_PATH_PREFIX;
import static org.globalart.astrum.blobstore.DirectPathLocationStrategy.DIRECT_PATH_ROOT;

/**
 * Tests for {@link DirectPathLocationStrategy}.
 */
public class DirectPathLocationStrategyTest
{
  private static final String CORRECT_PATH = "/healthCheckSummary/maven-central/current/summary.html";

  private static final String PATH_WITH_TRAVERSAL = "/healthCheckSummary/maven-central/1/../details/details.html";

  private static final String PATH_WITH_PREFIX_INSIDE_TRAVERSAL =
      "/healthCheckSummary/maven-central/1/.path$./details/details.html";

  private static final String EXPECTED_PATH = DIRECT_PATH_ROOT + "/" + CORRECT_PATH;

  private LocationStrategy underTest;

  @Before
  public void setUp() {
    underTest = new DirectPathLocationStrategy();
  }

  @Test
  public void testLocation() {
    String location = underTest.location(new BlobId(DIRECT_PATH_PREFIX + CORRECT_PATH));
    assertEquals(EXPECTED_PATH, location);
  }

  @Test(expected = IllegalArgumentException.class)
  public void testLocationWithTraversal() {
    underTest.location(new BlobId(DIRECT_PATH_PREFIX + PATH_WITH_TRAVERSAL));
  }

  @Test(expected = IllegalArgumentException.class)
  public void testLocationWithPrefixInsideTraversal() {
    underTest.location(new BlobId(DIRECT_PATH_PREFIX + PATH_WITH_PREFIX_INSIDE_TRAVERSAL));
  }

  @Test(expected = NullPointerException.class)
  public void testLocationWithNullableBlobId() {
    underTest.location(null);
  }
}
