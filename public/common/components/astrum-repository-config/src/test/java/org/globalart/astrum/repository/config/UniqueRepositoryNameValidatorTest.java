
package org.globalart.astrum.repository.config;

import org.globalart.astrum.repository.manager.RepositoryManager;

import org.junit.Test;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * Tests validity of Repository "foo"s validated by {@link UniqueRepositoryNameValidator}
 *
 * @since 3.0
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class UniqueRepositoryNameValidatorTest
{
  @Mock
  private RepositoryManager repositoryManager;

  @InjectMocks
  private UniqueRepositoryNameValidator validator;

  /*
   * Name is valid when the RepositoryManager says it does not exist
   */
  @Test
  public void testIsValid() {
    when(repositoryManager.exists("foo")).thenReturn(true, false);

    assertFalse(validator.isValid("foo", null));
    assertTrue(validator.isValid("foo", null));
  }
}
