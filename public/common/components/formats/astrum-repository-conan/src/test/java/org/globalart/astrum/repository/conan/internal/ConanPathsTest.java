package org.globalart.astrum.repository.conan.internal;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.conan.ConanCoords;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;

public class ConanPathsTest
{
  private final ConanCoords recipe = new ConanCoords("zlib", "1.3.1", "_", "_", "abc", null, null, null);

  @Test
  public void recipePaths() {
    assertThat(ConanPaths.recipeBase(recipe), is(equalTo("/zlib/1.3.1/_/_")));
    assertThat(ConanPaths.exportBase(recipe), is(equalTo("/zlib/1.3.1/_/_/abc/export/")));
    assertThat(ConanPaths.file(recipe.withFile("conanfile.py")), is(equalTo("/zlib/1.3.1/_/_/abc/export/conanfile.py")));
    assertThat(ConanPaths.metadata(ConanEndpoint.RECIPE_LATEST, recipe), is(equalTo("/zlib/1.3.1/_/_/latest.json")));
    assertThat(ConanPaths.metadata(ConanEndpoint.RECIPE_FILES, recipe),
        is(equalTo("/zlib/1.3.1/_/_/abc/files.json")));
  }

  @Test
  public void packagePaths() {
    ConanCoords pkg = recipe.withPackage("pid", "prev").withFile("conan_package.tgz");
    assertThat(ConanPaths.file(pkg), is(equalTo("/zlib/1.3.1/_/_/abc/package/pid/prev/conan_package.tgz")));
    assertThat(ConanPaths.metadata(ConanEndpoint.PACKAGE_REVISIONS, pkg),
        is(equalTo("/zlib/1.3.1/_/_/abc/package/pid/revisions.json")));
    assertThat(ConanPaths.metadata(ConanEndpoint.PACKAGE_FILES, pkg),
        is(equalTo("/zlib/1.3.1/_/_/abc/package/pid/prev/files.json")));
  }

  @Test
  public void v1PathsUseRevisionZero() {
    ConanCoords coords = new ConanCoords("zlib", "1.2.11", "_", "_", null, "pid", null, null);
    assertThat(ConanPaths.metadata(ConanEndpoint.V1_PACKAGE_DOWNLOAD_URLS, coords),
        is(equalTo("/zlib/1.2.11/_/_/0/package/pid/0/download_urls.json")));
    assertThat(ConanPaths.metadata(ConanEndpoint.V1_RECIPE_DIGEST, coords.withPackage(null, null)),
        is(equalTo("/zlib/1.2.11/_/_/0/digest.json")));
  }

  @Test
  public void namespace() {
    assertThat(recipe.namespace(), is(equalTo("")));
    assertThat(new ConanCoords("a", "1", "me", "_", null, null, null, null).namespace(), is(equalTo("me")));
    assertThat(new ConanCoords("a", "1", "me", "stable", null, null, null, null).namespace(),
        is(equalTo("me/stable")));
    assertThat(recipe.reference(), is(equalTo("zlib/1.3.1@_/_")));
  }

  @Test
  public void escapesLikeWildcards() {
    assertThat(ConanPaths.escapeLike("/a_b/50%/c\\d"), is(equalTo("/a\\_b/50\\%/c\\\\d")));
  }

  @Test(expected = InvalidContentException.class)
  public void rejectsTraversal() {
    recipe.withFile("../../etc/passwd").validate();
  }

  @Test(expected = InvalidContentException.class)
  public void rejectsBadSegment() {
    new ConanCoords("..", "1", null, null, null, null, null, null).validate();
  }

  @Test
  public void acceptsMetadataFiles() {
    recipe.withFile("metadata/logs/build.log").validate();
  }
}
