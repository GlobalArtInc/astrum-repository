
package org.globalart.astrum.blobstore.file.internal;

import java.io.File;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.file.FileBlobStore;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.rest.ValidationErrorsException;
import com.codahale.metrics.health.HealthCheck.Result;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.Mockito;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.blobstore.file.FileBlobStore.BASEDIR;
import static org.globalart.astrum.blobstore.file.FileBlobStore.CONFIG_KEY;
import static org.globalart.astrum.blobstore.file.FileBlobStore.PATH_KEY;

@RunWith(MockitoJUnitRunner.Silent.class)
public class FileBlobStorePathValidatorTest

{
  @Mock
  private BlobStoreManager blobStoreManager;

  @Mock
  private ApplicationDirectories applicationDirectories;

  List<BlobStore> blobStores;

  private FileBlobStorePathValidator underTest;

  @Before
  public void setUp() throws Exception {
    underTest = new FileBlobStorePathValidator(() -> blobStoreManager, () -> applicationDirectories);

    File appDir = Mockito.mock(File.class);
    when(applicationDirectories.getWorkDirectory(BASEDIR)).thenReturn(appDir);
    Path path = Mockito.mock(Path.class);
    when(path.toRealPath()).thenReturn(path);
    when(path.normalize()).thenReturn(Paths.get("/astrum"));
    when(appDir.toPath()).thenReturn(path);

    blobStores = generateBlobStores();
    when(blobStoreManager.browse()).thenReturn(blobStores);
  }

  @Test
  public void checkHealthyIfPathNotDuplicate() throws Exception {
    assertThat(underTest.check().isHealthy(), is(true));
  }

  @Test
  public void checkUnhealthyIfPathDuplicate() throws Exception {
    blobStores.add(generateBlobStore(FileBlobStore.TYPE, "f_d", "/astrum/0"));
    assertThat(underTest.check().isHealthy(), is(false));
  }

  @Test
  public void checkUnhealthyIfNestedPathDuplicate() throws Exception {
    blobStores.add(generateBlobStore(FileBlobStore.TYPE, "f_d", "/astrum/0/1"));
    Result result = underTest.check();
    assertThat(result.getMessage().contains("f_d"), is(true));
    assertThat(result.getMessage().contains("f0"), is(true));
    assertThat(result.isHealthy(), is(false));
  }

  @Test
  public void checkHealthyIfPathDuplicateInNorFileBlobStore() throws Exception {
    blobStores.add(generateBlobStore("other_type", "f_d", "/astrum/0"));
    assertThat(underTest.check().isHealthy(), is(true));
  }

  @Test
  public void notThrowsIfPathIsUnique() {
    BlobStoreConfiguration configuration =
        generateBlobStore(FileBlobStore.TYPE, "f_d", "/astrum/unique").getBlobStoreConfiguration();
    underTest.validatePathUniqueConstraint(configuration);
  }

  @Test(expected = ValidationErrorsException.class)
  public void throwsIfPathIsNotUnique() {
    BlobStoreConfiguration configuration =
        generateBlobStore(FileBlobStore.TYPE, "f_d", "/astrum/0").getBlobStoreConfiguration();
    underTest.validatePathUniqueConstraint(configuration);
  }

  @Test
  public void notThrowsIfPathIsNotUniqueInOtherBlobStoreType() {
    BlobStoreConfiguration configuration =
        generateBlobStore(FileBlobStore.TYPE, "f_d", "/astrum/10").getBlobStoreConfiguration();
    underTest.validatePathUniqueConstraint(configuration);
  }

  @Test(expected = ValidationErrorsException.class)
  public void throwsIfNotAbsolutePathIsNotUnique() {
    BlobStoreConfiguration configuration =
        generateBlobStore(FileBlobStore.TYPE, "f_d", "0").getBlobStoreConfiguration();
    underTest.validatePathUniqueConstraint(configuration);
  }

  private List<BlobStore> generateBlobStores() {
    blobStores = new ArrayList<>();

    blobStores.add(generateBlobStore("other_type", "s1", "/astrum/0"));
    blobStores.add(generateBlobStore("other_type", "s2", "/astrum/1"));
    blobStores.add(generateBlobStore("other_type", "s3", "/astrum/10"));

    for (int i = 0; i < 3; i++) {
      blobStores.add(generateBlobStore(FileBlobStore.TYPE, "f" + i, "/astrum/" + i));
    }

    blobStores.add(generateBlobStore(FileBlobStore.TYPE, "f00", "/astrum/00"));

    return blobStores;
  }

  private BlobStore generateBlobStore(final String type, final String name, final String path) {
    BlobStoreConfiguration configuration = Mockito.mock(BlobStoreConfiguration.class);
    when(configuration.getType()).thenReturn(type);
    when(configuration.getName()).thenReturn(name);

    Map<String, Object> map = new HashMap<>();
    map.put(PATH_KEY, path);
    NestedAttributesMap attributesMap = new NestedAttributesMap(CONFIG_KEY, map);
    when(configuration.attributes(CONFIG_KEY)).thenReturn(attributesMap);

    BlobStore blobStore = Mockito.mock(BlobStore.class);
    when(blobStore.getBlobStoreConfiguration()).thenReturn(configuration);

    return blobStore;
  }
}
