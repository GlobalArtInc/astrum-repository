package org.globalart.astrum.repository.rubygems.internal.gem;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Objects;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import javax.annotation.Nullable;

public record GemPlatform(@Nullable String cpu, @Nullable String os, @Nullable String version)
{
  public static final String RUBY = "ruby";

  private static final Pattern TRAILING_NUMBER = Pattern.compile("\\d+(\\.\\d+)?$");

  private static final Pattern NUMBER = Pattern.compile("^\\d+(\\.\\d+)?$");

  private static final Pattern X86 = Pattern.compile("i\\d86");

  private static final Pattern VALID = Pattern.compile("[A-Za-z0-9_.-]{1,64}");

  public static String normalize(@Nullable final String platform) {
    if (platform == null || platform.isBlank()) {
      return RUBY;
    }
    return platform.strip();
  }

  public static boolean isRuby(@Nullable final String platform) {
    return RUBY.equals(normalize(platform));
  }

  public static boolean isValid(@Nullable final String platform) {
    return platform == null || VALID.matcher(platform).matches();
  }

  public static GemPlatform parse(final String platform) {
    List<String> arch = new ArrayList<>(Arrays.asList(platform.split("-", -1)));
    if (arch.size() > 2 && !TRAILING_NUMBER.matcher(arch.get(arch.size() - 1)).find()) {
      String extra = arch.remove(arch.size() - 1);
      arch.set(arch.size() - 1, arch.get(arch.size() - 1) + "-" + extra);
    }
    String original = arch.remove(0);
    String cpu = X86.matcher(original).find() ? "x86" : original;
    if (arch.size() == 2 && NUMBER.matcher(arch.get(1)).matches()) {
      return new GemPlatform(cpu, arch.get(0), arch.get(1));
    }
    String os = arch.isEmpty() ? null : arch.get(0);
    if (os == null) {
      cpu = null;
      os = original;
    }
    return parseOs(cpu, os);
  }

  private static GemPlatform parseOs(@Nullable final String cpu, final String os) {
    Matcher m;
    if ((m = find("aix(\\d+)?", os)) != null) {
      return new GemPlatform(cpu, "aix", m.group(1));
    }
    if (find("cygwin", os) != null) {
      return new GemPlatform(cpu, "cygwin", null);
    }
    if ((m = find("darwin(\\d+)?", os)) != null) {
      return new GemPlatform(cpu, "darwin", m.group(1));
    }
    if (find("^macruby$", os) != null) {
      return new GemPlatform(cpu, "macruby", null);
    }
    if ((m = find("freebsd(\\d+)?", os)) != null) {
      return new GemPlatform(cpu, "freebsd", m.group(1));
    }
    if (find("^java$", os) != null || find("^jruby$", os) != null) {
      return new GemPlatform(cpu, "java", null);
    }
    if ((m = find("^java([\\d.]*)", os)) != null) {
      return new GemPlatform(cpu, "java", m.group(1));
    }
    if ((m = find("^dalvik(\\d+)?$", os)) != null) {
      return new GemPlatform(cpu, "dalvik", m.group(1));
    }
    if (find("^dotnet$", os) != null) {
      return new GemPlatform(cpu, "dotnet", null);
    }
    if ((m = find("^dotnet([\\d.]*)", os)) != null) {
      return new GemPlatform(cpu, "dotnet", m.group(1));
    }
    if ((m = find("linux-?(\\w+)?", os)) != null) {
      return new GemPlatform(cpu, "linux", m.group(1));
    }
    if (find("mingw32", os) != null) {
      return new GemPlatform(cpu, "mingw32", null);
    }
    if ((m = find("mingw-?(\\w+)?", os)) != null) {
      return new GemPlatform(cpu, "mingw", m.group(1));
    }
    if ((m = find("(mswin\\d+)(_(\\d+))?", os)) != null) {
      String name = m.group(1);
      return new GemPlatform(cpu == null && name.endsWith("32") ? "x86" : cpu, name, m.group(3));
    }
    if (find("netbsdelf", os) != null) {
      return new GemPlatform(cpu, "netbsdelf", null);
    }
    if ((m = find("openbsd(\\d+\\.\\d+)?", os)) != null) {
      return new GemPlatform(cpu, "openbsd", m.group(1));
    }
    if ((m = find("solaris(\\d+\\.\\d+)?", os)) != null) {
      return new GemPlatform(cpu, "solaris", m.group(1));
    }
    if (find("wasi", os) != null) {
      return new GemPlatform(cpu, "wasi", null);
    }
    if ((m = find("^(\\w+_platform)(\\d+)?", os)) != null) {
      return new GemPlatform(cpu, m.group(1), m.group(2));
    }
    return new GemPlatform(cpu, "unknown", null);
  }

  @Nullable
  private static Matcher find(final String regex, final String value) {
    Matcher matcher = Pattern.compile(regex).matcher(value);
    return matcher.find() ? matcher : null;
  }

  @Override
  public String toString() {
    return Stream.of(cpu, os, version).filter(Objects::nonNull).collect(Collectors.joining("-"));
  }
}
