
package org.globalart.astrum.repository.internal.blobstore;

import java.io.File;

import org.globalart.astrum.repository.manager.RepositoryManager;

import com.google.common.base.Strings;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.ExpectedException;
import org.junit.rules.TemporaryFolder;
import org.mockito.Mock;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BlobStoreUtilImplTest
{

  private static final int MAX_NAME_LENGTH = 255;

  private BlobStoreUtilImpl underTest;

  @Rule
  public TemporaryFolder temporaryFolder = new TemporaryFolder();

  @Rule
  public final ExpectedException exception = ExpectedException.none();

  @Mock
  private RepositoryManager repositoryManager;

  @Before
  public void setup() throws Exception {
    underTest = new BlobStoreUtilImpl(repositoryManager);
  }

  @Test
  public void testRequirePath() throws Exception {
    exception.expect(NullPointerException.class);
    underTest.validateFilePath(null, MAX_NAME_LENGTH);
  }

  @Test
  public void testRequirePositiveNonZeroLength() throws Exception {
    exception.expectMessage("maxLength should be greater than zero.");
    exception.expect(IllegalArgumentException.class);
    underTest.validateFilePath("test", 0);
  }

  @Test
  public void testAcceptShortPath() throws Exception {
    String shortPath = File.separator + "test" + File.separator + "test";
    String longMemberFolder = Strings.padEnd("", MAX_NAME_LENGTH, 'x');

    assertTrue(underTest.validateFilePath(shortPath, MAX_NAME_LENGTH));

    assertTrue(underTest.validateFilePath(shortPath + File.separator + "longer", 255));

    assertTrue(underTest.validateFilePath(shortPath + File.separator + longMemberFolder, 255));
  }

  @Test
  public void testRejectLongPath() throws Exception {
    String shortPath = File.separator + "test" + File.separator + "testtestte";

    assertFalse(underTest.validateFilePath(shortPath, 9));

    String longMemberFolder = Strings.padEnd("", MAX_NAME_LENGTH + 1, 'x');
    shortPath = File.separator + longMemberFolder + shortPath;

    assertFalse(underTest.validateFilePath(shortPath, MAX_NAME_LENGTH));
  }
}
