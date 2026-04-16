package org.globalart.astrum.repository.conda.internal.metadata;

import java.io.BufferedInputStream;
import java.io.BufferedOutputStream;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.SequenceInputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;

import org.globalart.astrum.repository.conda.AssetKind;

import com.fasterxml.jackson.core.JsonFactory;
import com.fasterxml.jackson.core.JsonGenerator;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonToken;
import com.fasterxml.jackson.core.StreamReadConstraints;
import com.fasterxml.jackson.databind.ObjectMapper;

import static java.nio.charset.StandardCharsets.UTF_8;

public final class Repodata
{
  public static final String PACKAGES = "packages";

  public static final String PACKAGES_CONDA = "packages.conda";

  public static final String REMOVED = "removed";

  private static final ObjectMapper MAPPER = new ObjectMapper();

  private static final JsonFactory FACTORY = JsonFactory.builder()
      .streamReadConstraints(StreamReadConstraints.builder()
          .maxStringLength(Integer.MAX_VALUE)
          .maxNestingDepth(10_000)
          .build())
      .build();

  private Repodata() {
  }

  public static byte[] build(final String subdir, final Collection<PackageRecord> records) throws IOException {
    Map<String, Object> packages = new TreeMap<>();
    Map<String, Object> condaPackages = new TreeMap<>();
    for (PackageRecord record : records) {
      Map<String, Object> target = record.assetKind == AssetKind.ARCH_CONDA_PACKAGE ? condaPackages : packages;
      target.put(record.fileName, record.fields);
    }
    Map<String, Object> repodata = new LinkedHashMap<>();
    repodata.put("info", Map.of("subdir", subdir));
    repodata.put(PACKAGES, packages);
    repodata.put(PACKAGES_CONDA, condaPackages);
    repodata.put(REMOVED, Collections.emptyList());
    repodata.put("repodata_version", 1);
    return MAPPER.writeValueAsBytes(repodata);
  }

  public static InputStream merge(final String subdir, final List<Source> sources) throws IOException {
    Path packagesFile = Files.createTempFile("conda-packages", ".json");
    Path condaPackagesFile = Files.createTempFile("conda-packages-conda", ".json");
    boolean success = false;
    try {
      Set<String> removed = new LinkedHashSet<>();
      try (JsonGenerator packages = FACTORY.createGenerator(
          new BufferedOutputStream(Files.newOutputStream(packagesFile)));
           JsonGenerator condaPackages = FACTORY.createGenerator(
               new BufferedOutputStream(Files.newOutputStream(condaPackagesFile)))) {
        packages.writeStartObject();
        condaPackages.writeStartObject();
        Set<String> seenPackages = new HashSet<>();
        Set<String> seenCondaPackages = new HashSet<>();
        for (Source source : sources) {
          try (InputStream in = new BufferedInputStream(source.open());
               JsonParser parser = FACTORY.createParser(in)) {
            copyMember(parser, packages, seenPackages, condaPackages, seenCondaPackages, removed);
          }
        }
        packages.writeEndObject();
        condaPackages.writeEndObject();
      }

      List<InputStream> parts = new ArrayList<>();
      parts.add(text("{\"info\":" + MAPPER.writeValueAsString(Map.of("subdir", subdir)) + ",\"" + PACKAGES + "\":"));
      parts.add(new DeletingFileInputStream(packagesFile));
      parts.add(text(",\"" + PACKAGES_CONDA + "\":"));
      parts.add(new DeletingFileInputStream(condaPackagesFile));
      parts.add(text(",\"" + REMOVED + "\":" + MAPPER.writeValueAsString(removed) + ",\"repodata_version\":1}"));
      success = true;
      return new SequenceInputStream(Collections.enumeration(parts));
    }
    finally {
      if (!success) {
        Files.deleteIfExists(packagesFile);
        Files.deleteIfExists(condaPackagesFile);
      }
    }
  }

  private static void copyMember(
      final JsonParser parser,
      final JsonGenerator packages,
      final Set<String> seenPackages,
      final JsonGenerator condaPackages,
      final Set<String> seenCondaPackages,
      final Set<String> removed) throws IOException
  {
    if (parser.nextToken() != JsonToken.START_OBJECT) {
      throw new IOException("Repodata is not a JSON object");
    }
    while (parser.nextToken() == JsonToken.FIELD_NAME) {
      String field = parser.currentName();
      JsonToken value = parser.nextToken();
      if (PACKAGES.equals(field) && value == JsonToken.START_OBJECT) {
        copyEntries(parser, packages, seenPackages);
      }
      else if (PACKAGES_CONDA.equals(field) && value == JsonToken.START_OBJECT) {
        copyEntries(parser, condaPackages, seenCondaPackages);
      }
      else if (REMOVED.equals(field) && value == JsonToken.START_ARRAY) {
        while (parser.nextToken() != JsonToken.END_ARRAY) {
          if (parser.currentToken() == JsonToken.VALUE_STRING) {
            removed.add(parser.getText());
          }
          else {
            parser.skipChildren();
          }
        }
      }
      else {
        parser.skipChildren();
      }
    }
  }

  private static void copyEntries(final JsonParser parser, final JsonGenerator target, final Set<String> seen)
      throws IOException
  {
    while (parser.nextToken() == JsonToken.FIELD_NAME) {
      String fileName = parser.currentName();
      parser.nextToken();
      if (seen.add(fileName)) {
        target.writeFieldName(fileName);
        target.copyCurrentStructure(parser);
      }
      else {
        parser.skipChildren();
      }
    }
  }

  private static InputStream text(final String value) {
    return new ByteArrayInputStream(value.getBytes(UTF_8));
  }

  public interface Source
  {
    InputStream open() throws IOException;
  }

  public static final class PackageRecord
  {
    private final String fileName;

    private final AssetKind assetKind;

    private final Map<String, Object> fields;

    public PackageRecord(final String fileName, final AssetKind assetKind, final Map<String, Object> fields) {
      this.fileName = fileName;
      this.assetKind = assetKind;
      this.fields = fields;
    }
  }

  private static final class DeletingFileInputStream
      extends InputStream
  {
    private final Path path;

    private InputStream delegate;

    private DeletingFileInputStream(final Path path) {
      this.path = path;
    }

    private InputStream delegate() throws IOException {
      if (delegate == null) {
        delegate = new BufferedInputStream(Files.newInputStream(path));
      }
      return delegate;
    }

    @Override
    public int read() throws IOException {
      return delegate().read();
    }

    @Override
    public int read(final byte[] b, final int off, final int len) throws IOException {
      return delegate().read(b, off, len);
    }

    @Override
    public void close() throws IOException {
      try {
        if (delegate != null) {
          delegate.close();
        }
      }
      finally {
        Files.deleteIfExists(path);
      }
    }
  }
}
