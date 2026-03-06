
package org.globalart.astrum.cleanup.storage.config;

import org.globalart.astrum.cleanup.storage.CleanupPolicyStorage;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class UniqueCleanupPolicyNameValidatorTest

{
  private UniqueCleanupPolicyNameValidator underTest;

  private static final String TEST_NAME = "test";

  @Mock
  private CleanupPolicyStorage cleanupPolicyStorage;

  @Before
  public void setUp() {
    underTest = new UniqueCleanupPolicyNameValidator(cleanupPolicyStorage);
  }

  @Test
  public void isValidByCleanupPolicyName() {
    when(cleanupPolicyStorage.exists(TEST_NAME)).thenReturn(false);

    assertThat(underTest.isValid(TEST_NAME, null), is(true));
  }

  @Test
  public void isInvalidByCleanupPolicyName() {
    when(cleanupPolicyStorage.exists(TEST_NAME)).thenReturn(true);

    assertThat(underTest.isValid(TEST_NAME, null), is(false));
  }
}
