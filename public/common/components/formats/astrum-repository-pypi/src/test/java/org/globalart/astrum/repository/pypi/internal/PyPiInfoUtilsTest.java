package org.globalart.astrum.repository.pypi.internal;

import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static java.util.Collections.singletonList;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

public class PyPiInfoUtilsTest
{
  private static final String DESCRIPTION = String.join("\n",
      "A sample Python project",
      "=======================",
      "",
      "A sample project that exists as an aid to the `Python Packaging User Guide",
      "<https://packaging.python.org>`_'s `Tutorial on Packaging and Distributing",
      "Projects <https://packaging.python.org/en/latest/distributing.html>`_.",
      "",
      "This projects does not aim to cover best practices for Python project",
      "development as a whole. For example, it does not provide guidance or tool",
      "recommendations for version control, documentation, or testing.",
      "",
      "----",
      "",
      "This is the README file for the project.",
      "",
      "The file should use UTF-8 encoding and be written using ReStructured Text. It",
      "will be used to generate the project webpage on PyPI and will be displayed as",
      "the project homepage on common code-hosting services, and should be written for",
      "that purpose.",
      "",
      "Typical contents for this file would include an overview of the project, basic",
      "usage examples, etc. Generally, including the project changelog in here is not",
      "a good idea, although a simple \"What's New\" section for the most recent version",
      "may be appropriate.",
      "");

  private static final String CLASSIFIERS = String.join("\n",
      "Development Status :: 3 - Alpha",
      "Intended Audience :: Developers",
      "Topic :: Software Development :: Build Tools",
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 2",
      "Programming Language :: Python :: 2.6",
      "Programming Language :: Python :: 2.7",
      "Programming Language :: Python :: 3",
      "Programming Language :: Python :: 3.3",
      "Programming Language :: Python :: 3.4",
      "Programming Language :: Python :: 3.5");

  private static Map<String, String> metadata(final String metadataVersion, final String archiveType) {
    Map<String, String> metadata = new LinkedHashMap<>();
    metadata.put("metadata_version", metadataVersion);
    metadata.put("name", "sample");
    metadata.put("version", "1.2.0");
    metadata.put("summary", "A sample Python project");
    metadata.put("home_page", "https://github.com/pypa/sampleproject");
    metadata.put("author", "The Python Packaging Authority");
    metadata.put("author_email", "pypa-dev@googlegroups.com");
    metadata.put("license", "MIT");
    metadata.put("description", DESCRIPTION);
    metadata.put("keywords", "sample setuptools development");
    metadata.put("platform", "UNKNOWN");
    metadata.put("classifiers", CLASSIFIERS);
    if ("2.0".equals(metadataVersion)) {
      metadata.put("provides_extra", "dev\ntest");
      metadata.put("requires_dist", "peppercorn\ncheck-manifest; extra == 'dev'\ncoverage; extra == 'test'");
    }
    metadata.put("archive_type", archiveType);
    return metadata;
  }

  private Map<String, String> extract(final String resource) throws Exception {
    try (InputStream in = getClass().getResourceAsStream(resource)) {
      return PyPiInfoUtils.extractMetadata(in);
    }
  }

  @Test
  public void extractsMetadataFromSourceArchives() throws Exception {
    assertEquals(metadata("1.1", "tar.bzip2"), extract("sample-1.2.0.tar.bz2"));
    assertEquals(metadata("1.1", "tar.gz"), extract("sample-1.2.0.tar.gz"));
    assertEquals(metadata("1.1", "zip"), extract("sample-1.2.0.zip"));
    assertEquals(metadata("1.1", "tar.z"), extract("sample-1.2.0.tar.Z"));
  }

  @Test
  public void extractsMetadataFromEggAndWheel() throws Exception {
    assertEquals(metadata("1.1", "zip"), extract("sample-1.2.0-py2.7.egg"));
    assertEquals(metadata("2.0", "zip"), extract("sample-1.2.0-py2.py3-none-any.whl"));
  }

  @Test
  public void parsesNonAsciiValues() throws Exception {
    try (InputStream in = getClass().getResourceAsStream("sample-metadata.txt")) {
      Map<String, List<String>> attributes = PyPiInfoUtils.parsePackageInfo(in);
      assertEquals(singletonList("Miguel Ángel García"), attributes.get("entrya"));
      assertEquals(singletonList("随机字符"), attributes.get("entryb"));
      assertEquals(singletonList("I'm just some normal, boring text!"), attributes.get("entryc"));
      assertEquals(singletonList("случайные символы"), attributes.get("entryi"));
      assertEquals(9, attributes.size());
    }
  }

  @Test
  public void readsDescriptionFromBody() throws Exception {
    String metadata = "Metadata-Version: 2.1\r\nName: My_Package\r\nVersion: 0.1.0\r\nClassifier: A\r\n"
        + "Classifier: B\r\nDescription-Content-Type: text/markdown\r\n\r\n  Body line\r\n second\r\n";
    Map<String, List<String>> attributes =
        PyPiInfoUtils.parsePackageInfo(new ByteArrayInputStream(metadata.getBytes(UTF_8)));
    assertEquals(singletonList("My_Package"), attributes.get("name"));
    assertEquals(List.of("A", "B"), attributes.get("classifiers"));
    assertEquals(singletonList("Body line\nsecond\n"), attributes.get("description"));
  }

  @Test
  public void returnsEmptyMapForUnreadableArchive() {
    assertTrue(PyPiInfoUtils.extractMetadata(new ByteArrayInputStream("not an archive".getBytes(UTF_8))).isEmpty());
  }
}
