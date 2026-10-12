package org.globalart.astrum.repository.yum.internal.rpm;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.zip.GZIPOutputStream;

import org.apache.commons.compress.archivers.cpio.CpioArchiveEntry;
import org.apache.commons.compress.archivers.cpio.CpioArchiveOutputStream;
import org.apache.commons.compress.archivers.cpio.CpioConstants;

public class TestRpm
{
  private final String name;

  private final String version;

  private final String release;

  private String arch = "x86_64";

  private Integer epoch;

  private boolean source;

  private String summary;

  private String description;

  private String license = "MIT";

  private String url;

  private String packager;

  private String vendor;

  private String group = "Unspecified";

  private long buildTime = 1700000000L;

  private final List<Object[]> files = new ArrayList<>();

  private final Map<Integer, List<Object[]>> dependencies = new LinkedHashMap<>();

  private final List<Object[]> changelogs = new ArrayList<>();

  public TestRpm(final String name, final String version, final String release) {
    this.name = name;
    this.version = version;
    this.release = release;
    this.summary = "The " + name + " package";
    this.description = "The " + name + " package for tests.";
  }

  public TestRpm arch(final String arch) {
    this.arch = arch;
    return this;
  }

  public TestRpm epoch(final int epoch) {
    this.epoch = epoch;
    return this;
  }

  public TestRpm source() {
    this.source = true;
    return this;
  }

  public TestRpm summary(final String summary) {
    this.summary = summary;
    return this;
  }

  public TestRpm description(final String description) {
    this.description = description;
    return this;
  }

  public TestRpm license(final String license) {
    this.license = license;
    return this;
  }

  public TestRpm url(final String url) {
    this.url = url;
    return this;
  }

  public TestRpm packager(final String packager) {
    this.packager = packager;
    return this;
  }

  public TestRpm vendor(final String vendor) {
    this.vendor = vendor;
    return this;
  }

  public TestRpm buildTime(final long buildTime) {
    this.buildTime = buildTime;
    return this;
  }

  public TestRpm file(final String path, final int size) {
    files.add(new Object[]{path, 0100644, 0, size});
    return this;
  }

  public TestRpm directory(final String path) {
    files.add(new Object[]{path, 040755, 0, 0});
    return this;
  }

  public TestRpm ghost(final String path) {
    files.add(new Object[]{path, 0100644, 1 << 6, 0});
    return this;
  }

  public TestRpm provides(final String dependency, final int flags, final String evr) {
    return dependency(RpmTags.PROVIDENAME, dependency, flags, evr);
  }

  public TestRpm requires(final String dependency, final int flags, final String evr) {
    return dependency(RpmTags.REQUIRENAME, dependency, flags, evr);
  }

  public TestRpm conflicts(final String dependency, final int flags, final String evr) {
    return dependency(RpmTags.CONFLICTNAME, dependency, flags, evr);
  }

  public TestRpm obsoletes(final String dependency, final int flags, final String evr) {
    return dependency(RpmTags.OBSOLETENAME, dependency, flags, evr);
  }

  public TestRpm recommends(final String dependency, final int flags, final String evr) {
    return dependency(RpmTags.RECOMMENDNAME, dependency, flags, evr);
  }

  public TestRpm changelog(final long time, final String author, final String text) {
    changelogs.add(new Object[]{time, author, text});
    return this;
  }

  private TestRpm dependency(final int nameTag, final String dependency, final int flags, final String evr) {
    dependencies.computeIfAbsent(nameTag, tag -> new ArrayList<>()).add(new Object[]{dependency, flags, evr});
    return this;
  }

  public String fileName() {
    return name + "-" + version + "-" + release + "." + (source ? "src" : arch) + ".rpm";
  }

  public byte[] build() throws IOException {
    byte[] payload = payload();
    HeaderWriter header = new HeaderWriter();
    header.string(RpmTags.NAME, name);
    header.string(RpmTags.VERSION, version);
    header.string(RpmTags.RELEASE, release);
    if (epoch != null) {
      header.int32(RpmTags.EPOCH, epoch);
    }
    header.i18n(RpmTags.SUMMARY, summary);
    header.i18n(RpmTags.DESCRIPTION, description);
    header.int32(RpmTags.BUILDTIME, (int) buildTime);
    header.string(RpmTags.BUILDHOST, "build.example.com");
    long installed = 0;
    for (Object[] file : files) {
      installed += (Integer) file[3];
    }
    header.int32(RpmTags.SIZE, (int) installed);
    if (vendor != null) {
      header.string(RpmTags.VENDOR, vendor);
    }
    header.string(RpmTags.LICENSE, license);
    if (packager != null) {
      header.string(RpmTags.PACKAGER, packager);
    }
    header.i18n(RpmTags.GROUP, group);
    if (url != null) {
      header.string(RpmTags.URL, url);
    }
    header.string(RpmTags.OS, "linux");
    header.string(RpmTags.ARCH, arch);
    if (source) {
      header.int32(RpmTags.SOURCEPACKAGE, 1);
    }
    else {
      header.string(RpmTags.SOURCERPM, name + "-" + version + "-" + release + ".src.rpm");
    }
    header.string(RpmTags.PAYLOADFORMAT, "cpio");
    header.string(RpmTags.PAYLOADCOMPRESSOR, "gzip");
    if (!files.isEmpty()) {
      List<String> dirNames = new ArrayList<>();
      List<String> baseNames = new ArrayList<>();
      int[] dirIndexes = new int[files.size()];
      int[] sizes = new int[files.size()];
      short[] modes = new short[files.size()];
      int[] flags = new int[files.size()];
      for (int i = 0; i < files.size(); i++) {
        String path = (String) files.get(i)[0];
        int slash = path.lastIndexOf('/');
        String dir = path.substring(0, slash + 1);
        if (!dirNames.contains(dir)) {
          dirNames.add(dir);
        }
        dirIndexes[i] = dirNames.indexOf(dir);
        baseNames.add(path.substring(slash + 1));
        modes[i] = (short) (int) (Integer) files.get(i)[1];
        flags[i] = (Integer) files.get(i)[2];
        sizes[i] = (Integer) files.get(i)[3];
      }
      header.int32(RpmTags.FILESIZES, sizes);
      header.int16(RpmTags.FILEMODES, modes);
      header.int32(RpmTags.FILEFLAGS, flags);
      header.int32(RpmTags.DIRINDEXES, dirIndexes);
      header.strings(RpmTags.BASENAMES, baseNames);
      header.strings(RpmTags.DIRNAMES, dirNames);
    }
    List<Object[]> provides = new ArrayList<>(dependencies.getOrDefault(RpmTags.PROVIDENAME, List.of()));
    if (!source) {
      provides.add(new Object[]{name, 8, (epoch == null ? "" : epoch + ":") + version + "-" + release});
      provides.add(new Object[]{name + "(x86-64)", 8, (epoch == null ? "" : epoch + ":") + version + "-" + release});
    }
    dependencyTags(header, provides, RpmTags.PROVIDENAME, RpmTags.PROVIDEFLAGS, RpmTags.PROVIDEVERSION);
    List<Object[]> requires = new ArrayList<>(dependencies.getOrDefault(RpmTags.REQUIRENAME, List.of()));
    requires.add(new Object[]{"rpmlib(CompressedFileNames)", 1 << 24 | 8 | 2, "3.0.4-1"});
    dependencyTags(header, requires, RpmTags.REQUIRENAME, RpmTags.REQUIREFLAGS, RpmTags.REQUIREVERSION);
    dependencyTags(header, dependencies.getOrDefault(RpmTags.CONFLICTNAME, List.of()), RpmTags.CONFLICTNAME,
        RpmTags.CONFLICTFLAGS, RpmTags.CONFLICTVERSION);
    dependencyTags(header, dependencies.getOrDefault(RpmTags.OBSOLETENAME, List.of()), RpmTags.OBSOLETENAME,
        RpmTags.OBSOLETEFLAGS, RpmTags.OBSOLETEVERSION);
    dependencyTags(header, dependencies.getOrDefault(RpmTags.RECOMMENDNAME, List.of()), RpmTags.RECOMMENDNAME,
        RpmTags.RECOMMENDFLAGS, RpmTags.RECOMMENDVERSION);
    if (!changelogs.isEmpty()) {
      int[] times = new int[changelogs.size()];
      List<String> authors = new ArrayList<>();
      List<String> texts = new ArrayList<>();
      for (int i = 0; i < changelogs.size(); i++) {
        times[i] = (int) (long) (Long) changelogs.get(i)[0];
        authors.add((String) changelogs.get(i)[1]);
        texts.add((String) changelogs.get(i)[2]);
      }
      header.int32(RpmTags.CHANGELOGTIME, times);
      header.strings(RpmTags.CHANGELOGNAME, authors);
      header.strings(RpmTags.CHANGELOGTEXT, texts);
    }
    byte[] headerBytes = header.toBytes();

    HeaderWriter signature = new HeaderWriter();
    signature.int32(RpmTags.SIGTAG_SIZE, headerBytes.length + payload.length);
    signature.int32(RpmTags.SIGTAG_PAYLOADSIZE, payloadArchiveSize);
    byte[] signatureBytes = signature.toBytes();

    ByteArrayOutputStream out = new ByteArrayOutputStream();
    out.write(lead());
    out.write(signatureBytes);
    out.write(new byte[(8 - signatureBytes.length % 8) % 8]);
    out.write(headerBytes);
    out.write(payload);
    return out.toByteArray();
  }

  private int payloadArchiveSize;

  private byte[] payload() throws IOException {
    ByteArrayOutputStream cpio = new ByteArrayOutputStream();
    try (CpioArchiveOutputStream archive = new CpioArchiveOutputStream(cpio, CpioConstants.FORMAT_NEW)) {
      for (Object[] file : files) {
        if (((Integer) file[2] & (1 << 6)) != 0) {
          continue;
        }
        CpioArchiveEntry entry = new CpioArchiveEntry(CpioConstants.FORMAT_NEW, "." + file[0]);
        int mode = (Integer) file[1];
        entry.setMode(mode);
        int size = (mode & 0170000) == 040000 ? 0 : (Integer) file[3];
        entry.setSize(size);
        archive.putArchiveEntry(entry);
        archive.write(new byte[size]);
        archive.closeArchiveEntry();
      }
    }
    payloadArchiveSize = cpio.size();
    ByteArrayOutputStream compressed = new ByteArrayOutputStream();
    try (GZIPOutputStream gzip = new GZIPOutputStream(compressed)) {
      gzip.write(cpio.toByteArray());
    }
    return compressed.toByteArray();
  }

  private byte[] lead() {
    byte[] lead = new byte[96];
    lead[0] = (byte) 0xED;
    lead[1] = (byte) 0xAB;
    lead[2] = (byte) 0xEE;
    lead[3] = (byte) 0xDB;
    lead[4] = 3;
    lead[5] = 0;
    lead[7] = (byte) (source ? 1 : 0);
    lead[9] = 1;
    byte[] nevr = (name + "-" + version + "-" + release).getBytes(StandardCharsets.UTF_8);
    System.arraycopy(nevr, 0, lead, 10, Math.min(nevr.length, 65));
    lead[77] = 1;
    lead[79] = 5;
    return lead;
  }

  private static void dependencyTags(
      final HeaderWriter header,
      final List<Object[]> entries,
      final int nameTag,
      final int flagsTag,
      final int versionTag)
  {
    if (entries.isEmpty()) {
      return;
    }
    List<String> names = new ArrayList<>();
    List<String> versions = new ArrayList<>();
    int[] flags = new int[entries.size()];
    for (int i = 0; i < entries.size(); i++) {
      names.add((String) entries.get(i)[0]);
      flags[i] = (Integer) entries.get(i)[1];
      versions.add((String) entries.get(i)[2]);
    }
    header.strings(nameTag, names);
    header.int32(flagsTag, flags);
    header.strings(versionTag, versions);
  }

  static final class HeaderWriter
  {
    private final TreeMap<Integer, Object[]> entries = new TreeMap<>();

    void string(final int tag, final String value) {
      entries.put(tag, new Object[]{RpmHeader.TYPE_STRING, List.of(value)});
    }

    void i18n(final int tag, final String value) {
      entries.put(tag, new Object[]{RpmHeader.TYPE_I18NSTRING, List.of(value)});
    }

    void strings(final int tag, final List<String> values) {
      entries.put(tag, new Object[]{RpmHeader.TYPE_STRING_ARRAY, values});
    }

    void int32(final int tag, final int... values) {
      entries.put(tag, new Object[]{RpmHeader.TYPE_INT32, values});
    }

    void int16(final int tag, final short... values) {
      entries.put(tag, new Object[]{RpmHeader.TYPE_INT16, values});
    }

    @SuppressWarnings("unchecked")
    byte[] toBytes() {
      ByteArrayOutputStream index = new ByteArrayOutputStream();
      ByteArrayOutputStream data = new ByteArrayOutputStream();
      for (Map.Entry<Integer, Object[]> entry : entries.entrySet()) {
        int type = (Integer) entry.getValue()[0];
        Object value = entry.getValue()[1];
        int alignment = type == RpmHeader.TYPE_INT32 ? 4 : type == RpmHeader.TYPE_INT16 ? 2 : 1;
        while (data.size() % alignment != 0) {
          data.write(0);
        }
        int offset = data.size();
        int count;
        if (value instanceof List<?> strings) {
          for (String string : (List<String>) strings) {
            data.writeBytes(string.getBytes(StandardCharsets.UTF_8));
            data.write(0);
          }
          count = strings.size();
        }
        else if (value instanceof int[] ints) {
          for (int number : ints) {
            writeInt(data, number);
          }
          count = ints.length;
        }
        else {
          short[] shorts = (short[]) value;
          for (short number : shorts) {
            data.write((number >> 8) & 0xff);
            data.write(number & 0xff);
          }
          count = shorts.length;
        }
        writeInt(index, entry.getKey());
        writeInt(index, type);
        writeInt(index, offset);
        writeInt(index, count);
      }
      ByteArrayOutputStream out = new ByteArrayOutputStream();
      writeInt(out, RpmHeader.MAGIC);
      writeInt(out, 0);
      writeInt(out, entries.size());
      writeInt(out, data.size());
      out.writeBytes(index.toByteArray());
      out.writeBytes(data.toByteArray());
      return out.toByteArray();
    }

    private static void writeInt(final ByteArrayOutputStream out, final int value) {
      out.write((value >> 24) & 0xff);
      out.write((value >> 16) & 0xff);
      out.write((value >> 8) & 0xff);
      out.write(value & 0xff);
    }
  }
}
