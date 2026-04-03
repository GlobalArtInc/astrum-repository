package org.globalart.astrum.repository.pypi.internal;

import java.io.InputStream;
import java.util.List;

import org.junit.Test;

import static java.util.Arrays.asList;
import static java.util.Collections.emptyList;
import static java.util.Collections.singletonList;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class PyPiIndexUtilsTest
{
  @Test
  public void extractsLinksFromIndexPage() throws Exception {
    try (InputStream in = getClass().getResourceAsStream("sample-index.html")) {
      assertEquals(asList(
          new PyPiLink("sample-1.2.0.tar.bz2", "/packages/sample-1.2.0.tar.bz2#md5=00c3db1c8ab5d10a2049fe384c8d53e5"),
          new PyPiLink("sample-1.2.1-py2.py3-none-any.whl",
              "/packages/sample-1.2.1-py2.py3-none-any.whl#md5=5c286195d47014fa0ba6e4e5b0801faf"),
          new PyPiLink("sample-1.2.1.tar.gz", "/packages/sample-1.2.1.tar.gz#md5=8c59858420df240b68e259ceae78a11d",
              ">=3.7")), PyPiIndexUtils.extractLinksFromIndex(in));
    }
  }

  @Test
  public void extractsNothingFromEmptyIndexPage() throws Exception {
    try (InputStream in = getClass().getResourceAsStream("sample-empty-index.html")) {
      assertEquals(emptyList(), PyPiIndexUtils.extractLinksFromIndex(in));
    }
  }

  @Test
  public void unescapesAttributesAndKeepsYankedMarker() {
    String html = "<a href=\"https://files.example/p/a-1.0.tar.gz#sha256=ab&amp;x\" data-requires-python=\"&gt;=3.8\" "
        + "data-yanked=\"broken &lt;build&gt;\">a-1.0.tar.gz</a>\n"
        + "<A HREF='https://files.example/p/a-1.1.tar.gz' data-yanked>a-1.1.tar.gz</A>\n"
        + "<a name=\"anchor\">no link</a>";
    assertEquals(asList(
        new PyPiLink("a-1.0.tar.gz", "https://files.example/p/a-1.0.tar.gz#sha256=ab&x", ">=3.8", "broken <build>"),
        new PyPiLink("a-1.1.tar.gz", "https://files.example/p/a-1.1.tar.gz", "", "")),
        PyPiIndexUtils.extractLinksFromIndex(html));
  }

  @Test
  public void rewritesPackageLinksToRepositoryPaths() {
    assertEquals(singletonList(new PyPiLink("sample-1.2.0.tar.bz2", "../../packages/sample/1.2.0/sample-1.2.0.tar.bz2", "3.7")),
        PyPiIndexUtils.makePackageLinksAstrumPaths("sample",
            singletonList(new PyPiLink("sample-1.2.0.tar.bz2", "../../packages/sample-1.2.0.tar.bz", "3.7"))));
    assertEquals(
        singletonList(new PyPiLink("sample-1.2.0.tar.bz2", "../../packages/sample/1.2.0/sample-1.2.0.tar.bz2#md5=00e5", "3.7")),
        PyPiIndexUtils.makePackageLinksAstrumPaths("sample",
            singletonList(new PyPiLink("sample-1.2.0.tar.bz2", "https://host/x/sample-1.2.0.tar.bz2#md5=00e5", "3.7"))));
  }

  @Test
  public void rewritesRootIndexLinksRelative() throws Exception {
    try (InputStream in = getClass().getResourceAsStream("pypi_index.htm")) {
      List<PyPiLink> links = PyPiIndexUtils.makeRootIndexLinksRelative(PyPiIndexUtils.extractLinksFromIndex(in));
      assertEquals(asList(
          new PyPiLink("1and1", "1and1/"),
          new PyPiLink("ansible-shell", "ansible-shell/"),
          new PyPiLink("roleplay", "roleplay/"),
          new PyPiLink("tibl-cli", "tibl-cli/"),
          new PyPiLink("zzhfun", "zzhfun/")), links);
    }
  }

  @Test
  public void validatesLinksAgainstNormalizedName() {
    List<PyPiLink> links = singletonList(new PyPiLink("My_Package-0.1.0.tar.gz", "x"));
    assertTrue(PyPiIndexUtils.validateIndexLinks("my-package", links));
    assertTrue(PyPiIndexUtils.validateIndexLinks("My.Package", links));
    assertFalse(PyPiIndexUtils.validateIndexLinks("other", links));
  }
}
