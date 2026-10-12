package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.repository.yum.internal.rpm.RpmPackage;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class RepodataBuilder
{
  private static final Logger log = LoggerFactory.getLogger(RepodataBuilder.class);

  private RepodataBuilder() {
  }

  public static Repodata build(final List<? extends PackageSource> sources, final long timestamp) throws IOException {
    List<PackageSource> current = new ArrayList<>(sources);
    List<String> skipped = new ArrayList<>();
    while (true) {
      List<PackageSource> failed = new ArrayList<>();
      try (MetadataFileWriter primary = new MetadataFileWriter(MetadataType.PRIMARY, current.size());
           MetadataFileWriter filelists = new MetadataFileWriter(MetadataType.FILELISTS, current.size());
           MetadataFileWriter other = new MetadataFileWriter(MetadataType.OTHER, current.size()))
      {
        for (PackageSource source : current) {
          RpmPackage rpm;
          try {
            rpm = source.read();
          }
          catch (IOException | IllegalArgumentException e) {
            log.warn("Leaving unreadable package {} out of the repodata: {}", source.location(), e.getMessage());
            failed.add(source);
            continue;
          }
          RepodataPackage repodataPackage = new RepodataPackage(rpm, source.location(), source.checksum(),
              source.size(), source.fileTime());
          primary.write(repodataPackage);
          filelists.write(repodataPackage);
          other.write(repodataPackage);
        }
        if (failed.isEmpty()) {
          List<GeneratedMetadata> files = new ArrayList<>();
          try {
            files.add(primary.finish(timestamp));
            files.add(filelists.finish(timestamp));
            files.add(other.finish(timestamp));
          }
          catch (IOException e) {
            for (GeneratedMetadata file : files) {
              file.close();
            }
            throw e;
          }
          Repomd repomd = new Repomd(Long.toString(timestamp), files.stream().map(GeneratedMetadata::data).toList());
          return new Repodata(repomd, files, current.size(), List.copyOf(skipped));
        }
      }
      current.removeAll(failed);
      failed.forEach(source -> skipped.add(source.location()));
    }
  }
}
