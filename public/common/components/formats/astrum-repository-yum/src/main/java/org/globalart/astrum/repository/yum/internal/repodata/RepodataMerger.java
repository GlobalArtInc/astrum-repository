package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.BitSet;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

import javax.xml.stream.XMLStreamConstants;
import javax.xml.stream.XMLStreamException;
import javax.xml.stream.XMLStreamReader;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class RepodataMerger
{
  private static final Logger log = LoggerFactory.getLogger(RepodataMerger.class);

  private static final Set<String> ROOTS = Set.of("metadata", "filelists", "otherdata");

  private RepodataMerger() {
  }

  public static Repodata merge(final List<? extends MemberRepodata> members, final long timestamp) throws IOException {
    Map<MemberRepodata, Selection> selections = new LinkedHashMap<>();
    Set<String> locations = new HashSet<>();
    int total = 0;
    for (MemberRepodata member : members) {
      Optional<RepomdData> primary = member.repomd().data(MetadataType.PRIMARY.type());
      if (primary.isEmpty()) {
        log.debug("Member {} has no primary metadata", member.name());
        continue;
      }
      Selection selection = select(member, primary.get(), locations);
      selections.put(member, selection);
      total += selection.packages.cardinality();
    }
    List<GeneratedMetadata> files = new ArrayList<>();
    try {
      try (MetadataFileWriter writer = new MetadataFileWriter(MetadataType.PRIMARY, total)) {
        for (Map.Entry<MemberRepodata, Selection> entry : selections.entrySet()) {
          copyPrimary(entry.getKey(), entry.getValue(), writer);
        }
        files.add(writer.finish(timestamp));
      }
      for (MetadataType type : List.of(MetadataType.FILELISTS, MetadataType.OTHER)) {
        try (MetadataFileWriter writer = new MetadataFileWriter(type, total)) {
          for (Map.Entry<MemberRepodata, Selection> entry : selections.entrySet()) {
            Optional<RepomdData> data = entry.getKey().repomd().data(type.type());
            if (data.isPresent()) {
              copyByPkgid(entry.getKey(), data.get(), entry.getValue().pkgids, writer);
            }
          }
          files.add(writer.finish(timestamp));
        }
      }
    }
    catch (IOException | RuntimeException e) {
      for (GeneratedMetadata file : files) {
        file.close();
      }
      throw e;
    }
    Repomd repomd = new Repomd(Long.toString(timestamp), files.stream().map(GeneratedMetadata::data).toList());
    return new Repodata(repomd, files, total, List.of());
  }

  private static Selection select(
      final MemberRepodata member,
      final RepomdData primary,
      final Set<String> locations) throws IOException
  {
    Selection selection = new Selection();
    try (InputStream in = member.open(primary)) {
      XMLStreamReader reader = RepodataXml.reader(in);
      try {
        int index = 0;
        while (nextPackage(reader)) {
          PackageKey key = readKey(reader);
          if (key.pkgid != null && locations.add(key.location)) {
            selection.packages.set(index);
            selection.pkgids.add(key.pkgid);
          }
          index++;
        }
      }
      finally {
        reader.close();
      }
    }
    catch (XMLStreamException e) {
      throw new IOException("Unreadable primary metadata of " + member.name() + ": " + e.getMessage(), e);
    }
    return selection;
  }

  private static void copyPrimary(
      final MemberRepodata member,
      final Selection selection,
      final MetadataFileWriter writer) throws IOException
  {
    RepomdData primary = member.repomd().data(MetadataType.PRIMARY.type()).orElseThrow();
    try (InputStream in = member.open(primary)) {
      XMLStreamReader reader = RepodataXml.reader(in);
      try {
        int index = 0;
        while (nextPackage(reader)) {
          if (selection.packages.get(index)) {
            writer.copy(reader);
          }
          else {
            RepodataXml.skipElement(reader);
          }
          index++;
        }
      }
      finally {
        reader.close();
      }
    }
    catch (XMLStreamException e) {
      throw new IOException("Unreadable primary metadata of " + member.name() + ": " + e.getMessage(), e);
    }
  }

  private static void copyByPkgid(
      final MemberRepodata member,
      final RepomdData data,
      final Set<String> pkgids,
      final MetadataFileWriter writer) throws IOException
  {
    Set<String> copied = new HashSet<>();
    try (InputStream in = member.open(data)) {
      XMLStreamReader reader = RepodataXml.reader(in);
      try {
        while (nextPackage(reader)) {
          String pkgid = reader.getAttributeValue(null, "pkgid");
          if (pkgid != null && pkgids.contains(pkgid) && copied.add(pkgid)) {
            writer.copy(reader);
          }
          else {
            RepodataXml.skipElement(reader);
          }
        }
      }
      finally {
        reader.close();
      }
    }
    catch (XMLStreamException e) {
      throw new IOException("Unreadable " + data.type() + " metadata of " + member.name() + ": " + e.getMessage(), e);
    }
  }

  private static boolean nextPackage(final XMLStreamReader reader) throws XMLStreamException {
    while (reader.hasNext()) {
      int event = reader.next();
      if (event != XMLStreamConstants.START_ELEMENT) {
        continue;
      }
      if ("package".equals(reader.getLocalName())) {
        return true;
      }
      if (!ROOTS.contains(reader.getLocalName())) {
        RepodataXml.skipElement(reader);
      }
    }
    return false;
  }

  private static PackageKey readKey(final XMLStreamReader reader) throws XMLStreamException {
    String pkgid = null;
    String location = null;
    int depth = 1;
    while (depth > 0 && reader.hasNext()) {
      int event = reader.next();
      if (event == XMLStreamConstants.START_ELEMENT) {
        depth++;
        if (depth == 2 && "checksum".equals(reader.getLocalName())) {
          pkgid = reader.getElementText().trim();
          depth--;
        }
        else if (depth == 2 && "location".equals(reader.getLocalName())) {
          String base = reader.getAttributeValue(RepodataXml.XML_NS, "base");
          location = (base == null ? "" : base + "|") + reader.getAttributeValue(null, "href");
        }
      }
      else if (event == XMLStreamConstants.END_ELEMENT) {
        depth--;
      }
    }
    return new PackageKey(pkgid, location == null ? pkgid : location);
  }

  private record PackageKey(String pkgid, String location)
  {
  }

  private static final class Selection
  {
    private final BitSet packages = new BitSet();

    private final Set<String> pkgids = new HashSet<>();
  }
}
