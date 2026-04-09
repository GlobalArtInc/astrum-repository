package org.globalart.astrum.repository.docker.internal.v2;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.mock;

public class DockerRouteMatcherTest
{
  private static final String DIGEST = "sha256:" + "a".repeat(64);

  private static Context context(final String action, final String path) {
    return new Context(mock(Repository.class), new Request.Builder().action(action).path(path).build());
  }

  private static DockerRoute match(
      final DockerRoute.Type type,
      final java.util.regex.Pattern pattern,
      final String action,
      final String path)
  {
    Context context = context(action, path);
    boolean matched = new DockerRouteMatcher(type, pattern, "GET", "HEAD", "PUT", "POST", "PATCH", "DELETE")
        .matches(context);
    return matched ? context.getAttributes().get(DockerRoute.class) : null;
  }

  @Test
  public void manifestByTagWithNestedName() {
    DockerRoute route = match(DockerRoute.Type.MANIFEST, DockerRouteMatcher.MANIFEST, "GET",
        "/v2/library/alpine/manifests/3.20");
    assertThat(route.getName(), is("library/alpine"));
    assertThat(route.getReference(), is("3.20"));
    assertThat(route.isDigestReference(), is(false));
  }

  @Test
  public void manifestByDigest() {
    DockerRoute route = match(DockerRoute.Type.MANIFEST, DockerRouteMatcher.MANIFEST, "HEAD",
        "/v2/a/b/c/manifests/" + DIGEST);
    assertThat(route.getName(), is("a/b/c"));
    assertThat(route.getReference(), is(DIGEST));
    assertThat(route.isDigestReference(), is(true));
  }

  @Test
  public void blobAndUploads() {
    assertThat(match(DockerRoute.Type.BLOB, DockerRouteMatcher.BLOB, "GET", "/v2/alpine/blobs/" + DIGEST)
        .getReference(), is(DIGEST));
    assertThat(match(DockerRoute.Type.UPLOADS, DockerRouteMatcher.UPLOADS, "POST", "/v2/my-app/blobs/uploads/")
        .getName(), is("my-app"));
    DockerRoute upload = match(DockerRoute.Type.UPLOAD, DockerRouteMatcher.UPLOAD, "PATCH",
        "/v2/my-app/blobs/uploads/0b5c7a8e-1d2f-4f9b-9c3e-2a1b0c9d8e7f");
    assertThat(upload.getName(), is("my-app"));
    assertThat(upload.getReference(), is("0b5c7a8e-1d2f-4f9b-9c3e-2a1b0c9d8e7f"));
  }

  @Test
  public void blobRouteDoesNotMatchUploads() {
    assertThat(match(DockerRoute.Type.BLOB, DockerRouteMatcher.BLOB, "GET", "/v2/alpine/blobs/uploads/"),
        nullValue());
  }

  @Test
  public void tagsCatalogAndBase() {
    assertThat(match(DockerRoute.Type.TAGS, DockerRouteMatcher.TAGS, "GET", "/v2/org/app/tags/list").getName(),
        is("org/app"));
    assertThat(match(DockerRoute.Type.CATALOG, DockerRouteMatcher.CATALOG, "GET", "/v2/_catalog").getType(),
        is(DockerRoute.Type.CATALOG));
    assertThat(match(DockerRoute.Type.BASE, DockerRouteMatcher.BASE, "GET", "/v2/").getType(),
        is(DockerRoute.Type.BASE));
  }

  @Test
  public void invalidNamesAndReferences() {
    assertThat(match(DockerRoute.Type.MANIFEST, DockerRouteMatcher.MANIFEST, "GET", "/v2/Alpine/manifests/1"),
        nullValue());
    assertThat(match(DockerRoute.Type.MANIFEST, DockerRouteMatcher.MANIFEST, "GET", "/v2/alpine/manifests/-bad"),
        nullValue());
    assertThat(DockerRouteMatcher.isValidName("library/alpine"), is(true));
    assertThat(DockerRouteMatcher.isValidName("UPPER/case"), is(false));
  }

  @Test
  public void actionIsChecked() {
    Context context = context("DELETE", "/v2/alpine/manifests/1");
    assertThat(new DockerRouteMatcher(DockerRoute.Type.MANIFEST, DockerRouteMatcher.MANIFEST, "GET", "HEAD")
        .matches(context), is(false));
  }
}
