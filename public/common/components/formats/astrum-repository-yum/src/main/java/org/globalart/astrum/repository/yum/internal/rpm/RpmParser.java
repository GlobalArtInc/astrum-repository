package org.globalart.astrum.repository.yum.internal.rpm;

import java.io.DataInputStream;
import java.io.EOFException;
import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;

import javax.annotation.Nullable;

import static org.globalart.astrum.repository.yum.internal.rpm.RpmDependency.PRE_MASK;
import static org.globalart.astrum.repository.yum.internal.rpm.RpmDependency.SENSE_RPMLIB;

public final class RpmParser
{
  public static final int LEAD_SIZE = 96;

  public static final int LEAD_MAGIC = 0xEDABEEDB;

  public static final int CHANGELOG_LIMIT = 10;

  private static final int FILE_TYPE_MASK = 0170000;

  private static final int FILE_TYPE_DIRECTORY = 0040000;

  private static final int FILE_FLAG_GHOST = 1 << 6;

  private static final String LIBC = "libc.so.6";

  private RpmParser() {
  }

  public static RpmPackage parse(final InputStream input) throws IOException {
    DataInputStream in = new DataInputStream(input);
    try {
      byte[] lead = new byte[LEAD_SIZE];
      in.readFully(lead);
      int magic = ((lead[0] & 0xff) << 24) | ((lead[1] & 0xff) << 16) | ((lead[2] & 0xff) << 8) | (lead[3] & 0xff);
      if (magic != LEAD_MAGIC) {
        throw new IllegalArgumentException("Not an RPM package");
      }
      RpmHeader signature = RpmHeader.read(in);
      long signatureSize = signature.size();
      int padding = (int) ((8 - (signatureSize % 8)) % 8);
      in.readFully(new byte[padding]);
      long headerStart = LEAD_SIZE + signatureSize + padding;
      RpmHeader header = RpmHeader.read(in);
      return build(header, signature, headerStart, headerStart + header.size());
    }
    catch (EOFException e) {
      throw new IllegalArgumentException("RPM package is truncated");
    }
  }

  static RpmPackage build(
      final RpmHeader header,
      final RpmHeader signature,
      final long headerStart,
      final long headerEnd)
  {
    String name = required(header, RpmTags.NAME, "name");
    String version = required(header, RpmTags.VERSION, "version");
    String release = required(header, RpmTags.RELEASE, "release");
    String epoch = header.number(RpmTags.EPOCH).map(String::valueOf).orElse("0");
    String arch = arch(header);
    List<RpmFile> files = files(header);
    List<RpmDependency> provides = dependencies(header, RpmTags.PROVIDENAME, RpmTags.PROVIDEFLAGS,
        RpmTags.PROVIDEVERSION);
    return new RpmPackage(
        name,
        epoch,
        version,
        release,
        arch,
        text(header, RpmTags.SUMMARY),
        text(header, RpmTags.DESCRIPTION),
        text(header, RpmTags.PACKAGER),
        text(header, RpmTags.URL),
        header.number(RpmTags.BUILDTIME).orElse(0L),
        header.number(RpmTags.LONGSIZE).or(() -> header.number(RpmTags.SIZE)).orElse(0L),
        archiveSize(header, signature),
        text(header, RpmTags.LICENSE),
        text(header, RpmTags.VENDOR),
        text(header, RpmTags.GROUP),
        text(header, RpmTags.BUILDHOST),
        text(header, RpmTags.SOURCERPM),
        headerStart,
        headerEnd,
        provides,
        requires(header, provides, files),
        dependencies(header, RpmTags.CONFLICTNAME, RpmTags.CONFLICTFLAGS, RpmTags.CONFLICTVERSION),
        dependencies(header, RpmTags.OBSOLETENAME, RpmTags.OBSOLETEFLAGS, RpmTags.OBSOLETEVERSION),
        dependencies(header, RpmTags.RECOMMENDNAME, RpmTags.RECOMMENDFLAGS, RpmTags.RECOMMENDVERSION),
        dependencies(header, RpmTags.SUGGESTNAME, RpmTags.SUGGESTFLAGS, RpmTags.SUGGESTVERSION),
        dependencies(header, RpmTags.SUPPLEMENTNAME, RpmTags.SUPPLEMENTFLAGS, RpmTags.SUPPLEMENTVERSION),
        dependencies(header, RpmTags.ENHANCENAME, RpmTags.ENHANCEFLAGS, RpmTags.ENHANCEVERSION),
        files,
        changelogs(header));
  }

  private static String required(final RpmHeader header, final int tag, final String what) {
    return header.string(tag)
        .filter(value -> !value.isBlank())
        .orElseThrow(() -> new IllegalArgumentException("RPM header has no " + what));
  }

  @Nullable
  private static String text(final RpmHeader header, final int tag) {
    return header.string(tag).orElse(null);
  }

  private static String arch(final RpmHeader header) {
    if (!header.has(RpmTags.SOURCERPM) || header.has(RpmTags.SOURCEPACKAGE)) {
      return header.has(RpmTags.NOSOURCE) || header.has(RpmTags.NOPATCH) ? "nosrc" : "src";
    }
    return required(header, RpmTags.ARCH, "architecture");
  }

  private static long archiveSize(final RpmHeader header, final RpmHeader signature) {
    return header.number(RpmTags.LONGARCHIVESIZE)
        .or(() -> header.number(RpmTags.ARCHIVESIZE))
        .or(() -> signature.number(RpmTags.SIGTAG_LONGARCHIVESIZE))
        .or(() -> signature.number(RpmTags.SIGTAG_PAYLOADSIZE))
        .orElse(0L);
  }

  private static List<RpmDependency> dependencies(
      final RpmHeader header,
      final int nameTag,
      final int flagsTag,
      final int versionTag)
  {
    List<String> names = header.strings(nameTag);
    if (names.isEmpty()) {
      return List.of();
    }
    long[] flags = header.numbers(flagsTag);
    List<String> versions = header.strings(versionTag);
    Set<RpmDependency> dependencies = new LinkedHashSet<>();
    for (int i = 0; i < names.size(); i++) {
      dependencies.add(RpmDependency.of(names.get(i), i < flags.length ? flags[i] : 0,
          i < versions.size() ? versions.get(i) : null, false));
    }
    return List.copyOf(dependencies);
  }

  private static List<RpmDependency> requires(
      final RpmHeader header,
      final List<RpmDependency> provides,
      final List<RpmFile> files)
  {
    List<String> names = header.strings(RpmTags.REQUIRENAME);
    if (names.isEmpty()) {
      return List.of();
    }
    long[] flags = header.numbers(RpmTags.REQUIREFLAGS);
    List<String> versions = header.strings(RpmTags.REQUIREVERSION);
    Set<String> primaryFiles = new HashSet<>();
    for (RpmFile file : files) {
      if (file.isPrimary()) {
        primaryFiles.add(file.path());
      }
    }
    Set<RpmDependency> provided = new HashSet<>(provides);
    Set<RpmDependency> requires = new LinkedHashSet<>();
    RpmDependency libc = null;
    for (int i = 0; i < names.size(); i++) {
      String name = names.get(i);
      long flag = i < flags.length ? flags[i] : 0;
      if (name.startsWith("rpmlib(") || (flag & SENSE_RPMLIB) != 0) {
        continue;
      }
      if (name.startsWith(LIBC)) {
        RpmDependency candidate = RpmDependency.of(name, flag, i < versions.size() ? versions.get(i) : null,
            (flag & PRE_MASK) != 0);
        if (libc == null || RpmVersions.compareLibcRequires(name, libc.name()) > 0) {
          libc = candidate;
        }
        continue;
      }
      if (name.startsWith("/") && primaryFiles.contains(name)) {
        continue;
      }
      RpmDependency dependency = RpmDependency.of(name, flag, i < versions.size() ? versions.get(i) : null,
          (flag & PRE_MASK) != 0);
      if (provided.contains(new RpmDependency(dependency.name(), dependency.flags(), dependency.epoch(),
          dependency.version(), dependency.release(), false))) {
        continue;
      }
      if (dependency.pre()) {
        requires.remove(withoutPre(dependency));
      }
      else if (requires.contains(withPre(dependency))) {
        continue;
      }
      requires.add(dependency);
    }
    if (libc != null) {
      requires.add(libc);
    }
    return List.copyOf(requires);
  }

  private static RpmDependency withPre(final RpmDependency dependency) {
    return new RpmDependency(dependency.name(), dependency.flags(), dependency.epoch(), dependency.version(),
        dependency.release(), true);
  }

  private static RpmDependency withoutPre(final RpmDependency dependency) {
    return new RpmDependency(dependency.name(), dependency.flags(), dependency.epoch(), dependency.version(),
        dependency.release(), false);
  }

  private static List<RpmFile> files(final RpmHeader header) {
    List<String> paths = new ArrayList<>();
    List<String> baseNames = header.strings(RpmTags.BASENAMES);
    if (!baseNames.isEmpty()) {
      List<String> dirNames = header.strings(RpmTags.DIRNAMES);
      long[] dirIndexes = header.numbers(RpmTags.DIRINDEXES);
      if (dirIndexes.length < baseNames.size()) {
        throw new IllegalArgumentException("RPM header has incomplete file directories");
      }
      for (int i = 0; i < baseNames.size(); i++) {
        long dirIndex = dirIndexes[i];
        if (dirIndex < 0 || dirIndex >= dirNames.size()) {
          throw new IllegalArgumentException("RPM header refers to a missing file directory");
        }
        paths.add(dirNames.get((int) dirIndex) + baseNames.get(i));
      }
    }
    else {
      paths.addAll(header.strings(RpmTags.OLDFILENAMES));
    }
    long[] modes = header.numbers(RpmTags.FILEMODES);
    long[] flags = header.numbers(RpmTags.FILEFLAGS);
    List<RpmFile> files = new ArrayList<>(paths.size());
    for (int i = 0; i < paths.size(); i++) {
      long mode = i < modes.length ? modes[i] : 0;
      long flag = i < flags.length ? flags[i] : 0;
      String type = RpmFile.FILE;
      if ((mode & FILE_TYPE_MASK) == FILE_TYPE_DIRECTORY) {
        type = RpmFile.DIR;
      }
      else if ((flag & FILE_FLAG_GHOST) != 0) {
        type = RpmFile.GHOST;
      }
      files.add(new RpmFile(paths.get(i), type));
    }
    return files;
  }

  private static List<RpmChangelog> changelogs(final RpmHeader header) {
    long[] times = header.numbers(RpmTags.CHANGELOGTIME);
    List<String> authors = header.strings(RpmTags.CHANGELOGNAME);
    List<String> texts = header.strings(RpmTags.CHANGELOGTEXT);
    int count = Math.min(CHANGELOG_LIMIT, Math.min(times.length, Math.min(authors.size(), texts.size())));
    List<RpmChangelog> changelogs = new ArrayList<>(count);
    for (int i = 0; i < count; i++) {
      changelogs.add(new RpmChangelog(authors.get(i), times[i], texts.get(i)));
    }
    Collections.reverse(changelogs);
    return changelogs;
  }
}
