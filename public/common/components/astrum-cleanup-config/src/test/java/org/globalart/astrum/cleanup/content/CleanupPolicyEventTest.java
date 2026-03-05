
package org.globalart.astrum.cleanup.content;

import org.globalart.astrum.cleanup.internal.storage.CleanupPolicyData;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.sameInstance;

/**
 * Tests for {@link CleanupPolicyEvent} and its concrete subclasses.
 */
public class CleanupPolicyEventTest
{
  private static final String POLICY_NAME = "test-cleanup-policy";

  private CleanupPolicyData cleanupPolicy;

  @Before
  public void setUp() {
    cleanupPolicy = new CleanupPolicyData();
    cleanupPolicy.setName(POLICY_NAME);
  }

  @Test
  public void testCreatedEventStoresPolicy() {
    CleanupPolicyCreatedEvent event = new CleanupPolicyCreatedEvent(cleanupPolicy);

    assertThat(event.getCleanupPolicy(), is(sameInstance(cleanupPolicy)));
  }

  @Test
  public void testUpdatedEventStoresPolicy() {
    CleanupPolicyUpdatedEvent event = new CleanupPolicyUpdatedEvent(cleanupPolicy);

    assertThat(event.getCleanupPolicy(), is(sameInstance(cleanupPolicy)));
  }

  @Test
  public void testDeletedEventStoresPolicy() {
    CleanupPolicyDeletedEvent event = new CleanupPolicyDeletedEvent(cleanupPolicy);

    assertThat(event.getCleanupPolicy(), is(sameInstance(cleanupPolicy)));
  }

  @Test
  public void testCreatedEventToString() {
    CleanupPolicyCreatedEvent event = new CleanupPolicyCreatedEvent(cleanupPolicy);

    String result = event.toString();
    assertThat(result, containsString("CleanupPolicyCreatedEvent"));
    assertThat(result, containsString("cleanupPolicy="));
    assertThat(result, containsString(cleanupPolicy.toString()));
  }

  @Test
  public void testUpdatedEventToString() {
    CleanupPolicyUpdatedEvent event = new CleanupPolicyUpdatedEvent(cleanupPolicy);

    String result = event.toString();
    assertThat(result, containsString("CleanupPolicyUpdatedEvent"));
    assertThat(result, containsString("cleanupPolicy="));
    assertThat(result, containsString(cleanupPolicy.toString()));
  }

  @Test
  public void testDeletedEventToString() {
    CleanupPolicyDeletedEvent event = new CleanupPolicyDeletedEvent(cleanupPolicy);

    String result = event.toString();
    assertThat(result, containsString("CleanupPolicyDeletedEvent"));
    assertThat(result, containsString("cleanupPolicy="));
    assertThat(result, containsString(cleanupPolicy.toString()));
  }

  @Test(expected = NullPointerException.class)
  public void testCreatedEventRejectsNullPolicy() {
    new CleanupPolicyCreatedEvent(null);
  }

  @Test(expected = NullPointerException.class)
  public void testUpdatedEventRejectsNullPolicy() {
    new CleanupPolicyUpdatedEvent(null);
  }

  @Test(expected = NullPointerException.class)
  public void testDeletedEventRejectsNullPolicy() {
    new CleanupPolicyDeletedEvent(null);
  }
}
